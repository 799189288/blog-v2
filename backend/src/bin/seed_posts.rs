// Seed a batch of posts for local development and demo screenshots.
// Usage:
//   cargo run --bin seed_posts                # 30 posts (default)
//   cargo run --bin seed_posts -- 60          # custom count
//
// Behaviour:
//   * All slugs are prefixed `seed-` so the seeded data is easy to
//     identify and wipe (`DELETE FROM posts WHERE slug LIKE 'seed-%'`).
//   * Idempotent: ON CONFLICT (slug) DO UPDATE re-renders content if
//     templates change. Re-running with a smaller count will leave the
//     extra posts from earlier runs in place — that's intentional.
//   * Author = first user in `users` (seed_admin first, otherwise this
//     bails with a clear error).
//   * Published posts get a `published_at` spread evenly across the
//     last ~180 days so the front page looks lived-in.

use std::collections::HashMap;
use std::env;

use anyhow::{Context, anyhow};
use sqlx::postgres::PgPoolOptions;
use time::{Duration, OffsetDateTime};

#[path = "../markdown.rs"]
mod markdown;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let count: usize = match env::args().nth(1) {
        Some(s) => s.parse().context("count must be a positive integer")?,
        None => 30,
    };
    if count == 0 {
        anyhow::bail!("count must be > 0");
    }

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    let author_id: i64 = sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT 1")
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| anyhow!("no users found — run `seed_admin` first"))?;

    let tag_ids = upsert_tags(&pool).await?;

    let templates = templates();
    let now = OffsetDateTime::now_utc();
    let span_days: i64 = 180;
    // step = how far apart consecutive posts are on the timeline, in seconds.
    // Older posts get older published_at; the newest one lands today.
    let step_secs = ((span_days * 24 * 3600) as f64 / count.max(1) as f64) as i64;

    let mut created = 0usize;
    let mut updated = 0usize;

    for i in 0..count {
        let tpl = &templates[i % templates.len()];
        let cycle = i / templates.len();
        let title = if cycle == 0 {
            tpl.title.to_string()
        } else {
            format!("{} #{}", tpl.title, cycle + 1)
        };

        // Make every cycle's slug unique by suffixing #cycle.
        let slug = format!("seed-{}", slug::slugify(&title));

        let content_md = tpl.body.to_string();
        let html = markdown::render(&content_md);
        let excerpt = markdown::excerpt(&content_md, 160);
        let word_count = markdown::word_count(&content_md) as i32;
        let reading_time = markdown::reading_time_min(&content_md);

        // Every 5th post is a draft so the admin list has variety.
        let is_draft = i % 5 == 4;
        let status = if is_draft { "draft" } else { "published" };
        let published_at: Option<OffsetDateTime> = if is_draft {
            None
        } else {
            // i=count-1 → newest (today); i=0 → oldest (~180d ago).
            Some(now - Duration::seconds(step_secs * (count as i64 - 1 - i as i64)))
        };

        // Upsert the post. `created_at` only set on INSERT so re-runs don't
        // shift it; `updated_at` and content fields refresh either way.
        // `xmax = 0` is Postgres's idiomatic "this row was inserted, not
        // updated, by the current statement" — lets us count create vs update.
        let row: (i64, bool) = sqlx::query_as(
            r#"
            INSERT INTO posts
                (slug, title, excerpt, content_md, content_html, status, author_id, published_at,
                 word_count, reading_time_min)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (slug) DO UPDATE SET
                title            = EXCLUDED.title,
                excerpt          = EXCLUDED.excerpt,
                content_md       = EXCLUDED.content_md,
                content_html     = EXCLUDED.content_html,
                status           = EXCLUDED.status,
                published_at     = EXCLUDED.published_at,
                word_count       = EXCLUDED.word_count,
                reading_time_min = EXCLUDED.reading_time_min,
                updated_at       = now()
            RETURNING id, (xmax = 0) AS inserted
            "#,
        )
        .bind(&slug)
        .bind(&title)
        .bind(&excerpt)
        .bind(&content_md)
        .bind(&html)
        .bind(status)
        .bind(author_id)
        .bind(published_at)
        .bind(word_count)
        .bind(reading_time)
        .fetch_one(&pool)
        .await?;
        let (post_id, inserted) = row;

        if inserted {
            created += 1;
        } else {
            updated += 1;
        }

        // Wipe & re-attach tag links so template tag changes propagate.
        sqlx::query("DELETE FROM post_tags WHERE post_id = $1")
            .bind(post_id)
            .execute(&pool)
            .await?;
        for tag_slug in tpl.tags {
            let tag_id = tag_ids
                .get(*tag_slug)
                .ok_or_else(|| anyhow!("missing tag in pool: {tag_slug}"))?;
            sqlx::query(
                "INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(post_id)
            .bind(*tag_id)
            .execute(&pool)
            .await?;
        }
    }

    println!(
        "seeded {count} posts: {created} created, {updated} updated. \
         author_id={author_id}. Clean up later with: \
         DELETE FROM posts WHERE slug LIKE 'seed-%';"
    );
    Ok(())
}

struct Template {
    title: &'static str,
    tags: &'static [&'static str],
    body: &'static str,
}

const TAG_POOL: &[(&str, &str)] = &[
    ("Rust", "rust"),
    ("Vue", "vue"),
    ("Frontend", "frontend"),
    ("PostgreSQL", "postgres"),
    ("Database", "database"),
    ("DevOps", "devops"),
    ("Docker", "docker"),
    ("Reading", "reading"),
    ("Life", "life"),
    ("Travel", "travel"),
    ("Music", "music"),
    ("Machine Learning", "ml"),
    ("Python", "python"),
    ("Security", "security"),
];

async fn upsert_tags(pool: &sqlx::PgPool) -> anyhow::Result<HashMap<&'static str, i64>> {
    let mut out = HashMap::new();
    for (name, slug) in TAG_POOL {
        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO tags (name, slug) VALUES ($1, $2)
            ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(name)
        .bind(slug)
        .fetch_one(pool)
        .await?;
        out.insert(*slug, id);
    }
    Ok(out)
}

fn templates() -> Vec<Template> {
    vec![
        Template {
            title: "Rust 所有权:从借用检查器学到的事",
            tags: &["rust"],
            body: r#"## 引子

第一次写 Rust 的人,十有八九都会被借用检查器骂。但写久了你会发现,它骂的那些点,在其他语言里恰恰是 bug 的高发区。

## 一个最小的例子

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

生命周期标注看着繁琐,但它精确表达了"返回值的存活时间不能超过任一参数"——这本来就是事实,C++ 里这种事实是隐式的,出了问题就是 dangling reference。

## 我的几条经验

- **先写函数签名再写实现**。生命周期标注是接口的一部分,实现可以随便改。
- **`Cow<'_, str>` 比想象中常用**。需要"可能借用也可能拥有"时不要立刻 `to_string()`。
- **`&str` 优先于 `String`**。除非真的需要所有权。

## 结语

借用检查器不是敌人,它是一个永不疲倦的 code reviewer。
"#,
        },
        Template {
            title: "Vue 3 组合式 API 的几个反模式",
            tags: &["vue", "frontend"],
            body: r#"## 背景

Composition API 给了我们更大的自由,但自由意味着更容易写出乱七八糟的代码。这里记录几个我和团队都踩过的坑。

## 反模式 1: 把所有逻辑塞进 setup

```vue
<script setup>
// 200 行的 setup
</script>
```

setup 不是 mounted。复杂逻辑该抽成 composable 就抽,不要在 setup 里硬堆。

## 反模式 2: 滥用 ref 包装对象

`ref({ a: 1, b: 2 })` 完全可以写成 `reactive({ a: 1, b: 2 })`,后者访问字段不需要 `.value`。

## 反模式 3: watch 的 deep + 大对象

deep watch 一个深层结构的对象,性能直接劣化。能用 computed 派生就用 computed。

## 小结

新工具不会自动带来好代码,只会放大你已有的习惯——好的更好,坏的更糟。
"#,
        },
        Template {
            title: "PostgreSQL 索引:什么时候不该建索引",
            tags: &["postgres", "database"],
            body: r#"## 反直觉的事实

不是所有查询都该建索引。索引不是免费的:写入要维护、占磁盘、占内存。

## 不该建索引的场景

1. **小表**(< 1000 行):全表扫比走索引快
2. **写多读少**:每次 INSERT/UPDATE 都要更新索引
3. **选择性低的列**(比如布尔值):索引可能比全表扫还慢
4. **频繁全表扫描的分析查询**:索引帮不上忙

## EXPLAIN 才是真理

```sql
EXPLAIN (ANALYZE, BUFFERS) SELECT ...;
```

看 `Seq Scan` vs `Index Scan`,看 `cost` 和 `actual time`。不要凭感觉建索引。

## 部分索引是被低估的工具

```sql
CREATE INDEX ON orders (created_at) WHERE status = 'pending';
```

只索引"还没处理的订单",占用小、命中率高。
"#,
        },
        Template {
            title: "Docker Compose 部署个人项目的几个坑",
            tags: &["devops", "docker"],
            body: r#"## 写在前面

把一个小项目用 docker-compose 部到自己的 VPS 上,看起来五分钟搞定,实际上经常半天填坑。

## 坑 1: 数据库数据没持久化

```yaml
services:
  db:
    image: postgres:16
    # 没有 volumes —— 重启数据全没
```

一定要挂 named volume,不要用匿名卷。

## 坑 2: 服务启动顺序

`depends_on` 只保证启动顺序,不保证服务可用。Postgres 进程起来不代表能接受连接。要么后端加重试,要么用 `healthcheck` + `depends_on.condition: service_healthy`。

## 坑 3: 把 .env 提交到 git

`.gitignore` 里 `.env`,提供一份 `.env.example`。这条没什么技术含量,但出过事的人都记得很牢。

## 一句话总结

部署的 80% 工作是"让它在出错时还能恢复",而不是"让它跑起来"。
"#,
        },
        Template {
            title: "读书笔记:《Designing Data-Intensive Applications》",
            tags: &["reading"],
            body: r#"## 总评

一本"反碎片化"的书。同样讲分布式系统,网上博客是一堆点,这本书把它们连成一张图。

## 我最有共鸣的几章

- **第 5 章 复制**:Leader-follower 看似简单,但故障切换是无穷的细节
- **第 7 章 事务**:隔离级别不是越高越好,大多数业务在 Read Committed 下就够用
- **第 9 章 一致性与共识**:Linearizability、Causal、Sequential 几个一致性模型的对比清晰得像教科书

## 一个改变我看法的观点

> "Eventual consistency" 这个词太弱了,它没告诉你"最终"是多久。

作者更喜欢"convergent"——强调"会收敛"而不是"会延迟"。

## 行动项

- 写存储相关的代码前先想一遍:这次操作的一致性诉求到底有多强?
- 不要默认上分布式事务,先看能不能用幂等 + 重试解决。
"#,
        },
        Template {
            title: "周末记:把咖啡机拆了又装回去",
            tags: &["life"],
            body: r#"周六醒来发现咖啡机不出水了。

按提示除垢、清洗,折腾两小时还是不行。索性把外壳拧开,看清楚水路怎么走的——结果发现是一根硅胶管被水垢顶变形了。

剪掉变形的一截,接回去,咖啡机又活了。

修家电这种事,一半是手艺,一半是耐心。每次都告诉自己下次再坏直接换,然后下次又把它拆开。

下午去附近书店翻了一会儿书,买了本不打算立刻读的小说。回家路上买了菜,做了个味噌汤。

普通的一天,但比加班舒服一万倍。
"#,
        },
        Template {
            title: "去了一趟京都:那些攻略不会写的细节",
            tags: &["travel"],
            body: r#"## 关于交通

地铁卡(ICOCA)直接在便利店充值就行,不用专门去车站。但如果是从关西机场进京都,直接买"HARUKA + ICOCA 套票"更划算。

## 关于吃

- 拉面在京都不是强项。想吃面的话,关西的乌冬反而更地道。
- 锦市场是观光价,本地人不去。
- 真正好吃的家常店往往没招牌,Google Maps 上评论很少但分高的那种就对了。

## 关于寺院

- 早上 8 点到清水寺,跟下午 3 点是两个世界。
- 苔寺需要提前预约,而且要抄经。值得去一次。

## 关于钱

现金还是要带的。小店、地藏王、神社的赛钱箱都吃硬币。
"#,
        },
        Template {
            title: "最近循环的几张专辑",
            tags: &["music"],
            body: r#"## 最近循环的几张

- **Phoebe Bridgers — Punisher**:有一种"伤口结痂之前那几天"的氛围
- **Black Country, New Road — Ants From Up There**:整张听完像看了一场话剧
- **Khruangbin — Mordechai**:做事时背景音的最佳选择,有人声但不打扰

## 一个被低估的习惯

每天上下班路上专心听一张专辑,不切歌。比刷碎片化短视频回血得多。

## 一首推荐

Phoebe Bridgers 的 "Moon Song",副歌一句:

> And if I could give you the moon, I would give you the moon.

简单到像废话,但放在那个旋律里就不是废话。
"#,
        },
        Template {
            title: "机器学习入门:为什么 Andrew Ng 的课依然值得看",
            tags: &["ml", "python"],
            body: r#"## 背景

虽然 2024 年才入门 ML 听起来有点晚,但 Andrew Ng 的 Coursera 课依然是我推荐的第一门。

## 它好在哪

- **不跳步骤**。线性回归的梯度下降一行一行推给你看。
- **数学只在必要时出现**,不会一上来就甩你 100 页矩阵推导。
- **作业实操**。手写一遍梯度下降比看十遍理论都管用。

## 配合什么看

- 看完课配合《Hands-On Machine Learning》(O'Reilly 那本)动手做项目
- Kaggle 找一个 getting started 比赛,从头跑通一遍 baseline

## 小心的几个误区

1. 把 ML 等同于 deep learning。99% 的业务问题,XGBoost 就够了。
2. 沉迷调参,不重视特征工程。
3. 用 accuracy 作为唯一指标。类别不平衡时它毫无意义。
"#,
        },
        Template {
            title: "给小项目做基本安全审计的 checklist",
            tags: &["security"],
            body: r#"## 谁需要这份 checklist

不是给 enterprise 用的,是给"个人博客、副业项目、小工具"用的——你不会上专门的安全团队,但也不想哪天被人拿去挖矿。

## Web 服务

- [ ] HTTPS,不用 http
- [ ] 关掉 server header 的版本号(Nginx 的 `server_tokens off`)
- [ ] 限制 CORS 来源,不要 `*`
- [ ] 至少一个简单的 rate limit(每 IP 每分钟 N 次)

## 认证

- [ ] 密码用 Argon2 或 bcrypt,不要 MD5/SHA1
- [ ] JWT secret 长度 >= 32 字节
- [ ] token 过期时间合理(24h 或更短)
- [ ] 登录失败不区分"用户名不存在"和"密码错误"

## 数据库

- [ ] 业务账号不要用 superuser
- [ ] 数据库端口不暴露公网
- [ ] 用预编译参数,杜绝 SQL 注入(sqlx/sea-orm 默认就是)
- [ ] 备份策略,且定期演练恢复

## 服务器

- [ ] SSH 关密码登录,只允许 key
- [ ] fail2ban 装一下
- [ ] 自动安全更新(unattended-upgrades)
- [ ] 防火墙只开必要的端口

## 一句话

安全是个工程问题,不是一次性任务。每次部署前过一遍这个 checklist,90% 的低级错误就能避免。
"#,
        },
    ]
}
