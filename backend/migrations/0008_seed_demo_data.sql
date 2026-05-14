-- Seed demo data: admin user, tags, and sample published posts.
--
-- This migration is idempotent (ON CONFLICT DO NOTHING / DO UPDATE) so it
-- is safe to run against a database that was already partially seeded by
-- the `seed_admin` / `seed_posts` binaries.
--
-- Password for the "admin" user is "admin123456".
-- Hash was produced by argon2 v0.5 (Argon2id, m=19456, t=2, p=1) —
-- the same parameters used by `src/auth/password.rs` via `Argon2::default()`.
--
-- To rotate the password after first deploy, run:
--   cargo run --bin seed_admin -- admin <new-password>

-- ─── Admin user ──────────────────────────────────────────────────────────────
-- Password: admin123456
--
-- Hash produced by argon2 v0.5 (Argon2id, m=19456, t=2, p=1) — the exact
-- parameters emitted by `Argon2::default()` in src/auth/password.rs.
--
-- To verify or rotate after first deploy:
--   cargo run --bin seed_admin -- admin <new-password>
--
-- To generate a fresh hash independently (requires argon2-cffi or similar):
--   python3 -c "
--     from argon2 import PasswordHasher
--     ph = PasswordHasher(time_cost=2, memory_cost=19456, parallelism=1)
--     print(ph.hash('admin123456'))
--   "

INSERT INTO users (username, password_hash, role)
VALUES (
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno',
    'admin'
)
ON CONFLICT (username) DO NOTHING;

-- ─── Tags ─────────────────────────────────────────────────────────────────────
-- Mirrors TAG_POOL in src/bin/seed_posts.rs exactly so the two seeding
-- paths stay in sync.

INSERT INTO tags (name, slug) VALUES
    ('Rust',             'rust'),
    ('Vue',              'vue'),
    ('Frontend',         'frontend'),
    ('PostgreSQL',       'postgres'),
    ('Database',         'database'),
    ('DevOps',           'devops'),
    ('Docker',           'docker'),
    ('Reading',          'reading'),
    ('Life',             'life'),
    ('Travel',           'travel'),
    ('Music',            'music'),
    ('Machine Learning', 'ml'),
    ('Python',           'python'),
    ('Security',         'security')
ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name;

-- ─── Sample posts ─────────────────────────────────────────────────────────────
-- 9 published posts spread across the last ~180 days.
-- Slugs are prefixed "demo-" (distinct from the "seed-" prefix used by the
-- seed_posts binary) so they can be identified and removed independently:
--   DELETE FROM posts WHERE slug LIKE 'demo-%';
--
-- content_html is a minimal rendered version of content_md; the application
-- re-renders on edit so this only needs to be good enough for first display.

INSERT INTO posts (slug, title, excerpt, content_md, content_html,
                   status, author_id, published_at, word_count, reading_time_min)
VALUES

-- 1. Rust ownership
(
    'demo-rust-ownership-borrow-checker',
    'Rust 所有权:从借用检查器学到的事',
    '第一次写 Rust 的人,十有八九都会被借用检查器骂。但写久了你会发现,它骂的那些点,在其他语言里恰恰是 bug 的高发区。',
    E'## 引子\n\n第一次写 Rust 的人,十有八九都会被借用检查器骂。但写久了你会发现,它骂的那些点,在其他语言里恰恰是 bug 的高发区。\n\n## 一个最小的例子\n\n```rust\nfn longest<\'a>(x: &\'a str, y: &\'a str) -> &\'a str {\n    if x.len() > y.len() { x } else { y }\n}\n```\n\n生命周期标注看着繁琐,但它精确表达了"返回值的存活时间不能超过任一参数"——这本来就是事实,C++ 里这种事实是隐式的,出了问题就是 dangling reference。\n\n## 我的几条经验\n\n- **先写函数签名再写实现**。生命周期标注是接口的一部分,实现可以随便改。\n- **`Cow<\'_, str>` 比想象中常用**。需要"可能借用也可能拥有"时不要立刻 `to_string()`。\n- **`&str` 优先于 `String`**。除非真的需要所有权。\n\n## 结语\n\n借用检查器不是敌人,它是一个永不疲倦的 code reviewer。\n',
    '<h2>引子</h2><p>第一次写 Rust 的人,十有八九都会被借用检查器骂。但写久了你会发现,它骂的那些点,在其他语言里恰恰是 bug 的高发区。</p><h2>一个最小的例子</h2><pre><code class="language-rust">fn longest&lt;''a&gt;(x: &amp;''a str, y: &amp;''a str) -&gt; &amp;''a str {\n    if x.len() &gt; y.len() { x } else { y }\n}</code></pre><p>生命周期标注看着繁琐,但它精确表达了"返回值的存活时间不能超过任一参数"——这本来就是事实,C++ 里这种事实是隐式的,出了问题就是 dangling reference。</p><h2>我的几条经验</h2><ul><li><strong>先写函数签名再写实现</strong>。生命周期标注是接口的一部分,实现可以随便改。</li><li><strong><code>Cow&lt;''_, str&gt;</code> 比想象中常用</strong>。需要"可能借用也可能拥有"时不要立刻 <code>to_string()</code>。</li><li><strong><code>&amp;str</code> 优先于 <code>String</code></strong>。除非真的需要所有权。</li></ul><h2>结语</h2><p>借用检查器不是敌人,它是一个永不疲倦的 code reviewer。</p>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '160 days',
    180,
    1
),

-- 2. Vue 3 anti-patterns
(
    'demo-vue3-composition-api-antipatterns',
    'Vue 3 组合式 API 的几个反模式',
    'Composition API 给了我们更大的自由,但自由意味着更容易写出乱七八糟的代码。这里记录几个我和团队都踩过的坑。',
    E'## 背景\n\nComposition API 给了我们更大的自由,但自由意味着更容易写出乱七八糟的代码。这里记录几个我和团队都踩过的坑。\n\n## 反模式 1: 把所有逻辑塞进 setup\n\n```vue\n<script setup>\n// 200 行的 setup\n</script>\n```\n\nsetup 不是 mounted。复杂逻辑该抽成 composable 就抽,不要在 setup 里硬堆。\n\n## 反模式 2: 滥用 ref 包装对象\n\n`ref({ a: 1, b: 2 })` 完全可以写成 `reactive({ a: 1, b: 2 })`,后者访问字段不需要 `.value`。\n\n## 反模式 3: watch 的 deep + 大对象\n\ndeep watch 一个深层结构的对象,性能直接劣化。能用 computed 派生就用 computed。\n\n## 小结\n\n新工具不会自动带来好代码,只会放大你已有的习惯——好的更好,坏的更糟。\n',
    '<h2>背景</h2><p>Composition API 给了我们更大的自由,但自由意味着更容易写出乱七八糟的代码。这里记录几个我和团队都踩过的坑。</p><h2>反模式 1: 把所有逻辑塞进 setup</h2><pre><code class="language-vue">&lt;script setup&gt;\n// 200 行的 setup\n&lt;/script&gt;</code></pre><p>setup 不是 mounted。复杂逻辑该抽成 composable 就抽,不要在 setup 里硬堆。</p><h2>反模式 2: 滥用 ref 包装对象</h2><p><code>ref({ a: 1, b: 2 })</code> 完全可以写成 <code>reactive({ a: 1, b: 2 })</code>,后者访问字段不需要 <code>.value</code>。</p><h2>反模式 3: watch 的 deep + 大对象</h2><p>deep watch 一个深层结构的对象,性能直接劣化。能用 computed 派生就用 computed。</p><h2>小结</h2><p>新工具不会自动带来好代码,只会放大你已有的习惯——好的更好,坏的更糟。</p>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '140 days',
    160,
    1
),

-- 3. PostgreSQL indexes
(
    'demo-postgres-when-not-to-index',
    'PostgreSQL 索引:什么时候不该建索引',
    '不是所有查询都该建索引。索引不是免费的:写入要维护、占磁盘、占内存。',
    E'## 反直觉的事实\n\n不是所有查询都该建索引。索引不是免费的:写入要维护、占磁盘、占内存。\n\n## 不该建索引的场景\n\n1. **小表**(< 1000 行):全表扫比走索引快\n2. **写多读少**:每次 INSERT/UPDATE 都要更新索引\n3. **选择性低的列**(比如布尔值):索引可能比全表扫还慢\n4. **频繁全表扫描的分析查询**:索引帮不上忙\n\n## EXPLAIN 才是真理\n\n```sql\nEXPLAIN (ANALYZE, BUFFERS) SELECT ...;\n```\n\n看 `Seq Scan` vs `Index Scan`,看 `cost` 和 `actual time`。不要凭感觉建索引。\n\n## 部分索引是被低估的工具\n\n```sql\nCREATE INDEX ON orders (created_at) WHERE status = ''pending'';\n```\n\n只索引"还没处理的订单",占用小、命中率高。\n',
    '<h2>反直觉的事实</h2><p>不是所有查询都该建索引。索引不是免费的:写入要维护、占磁盘、占内存。</p><h2>不该建索引的场景</h2><ol><li><strong>小表</strong>(&lt; 1000 行):全表扫比走索引快</li><li><strong>写多读少</strong>:每次 INSERT/UPDATE 都要更新索引</li><li><strong>选择性低的列</strong>(比如布尔值):索引可能比全表扫还慢</li><li><strong>频繁全表扫描的分析查询</strong>:索引帮不上忙</li></ol><h2>EXPLAIN 才是真理</h2><pre><code class="language-sql">EXPLAIN (ANALYZE, BUFFERS) SELECT ...;</code></pre><p>看 <code>Seq Scan</code> vs <code>Index Scan</code>,看 <code>cost</code> 和 <code>actual time</code>。不要凭感觉建索引。</p><h2>部分索引是被低估的工具</h2><pre><code class="language-sql">CREATE INDEX ON orders (created_at) WHERE status = ''pending'';</code></pre><p>只索引"还没处理的订单",占用小、命中率高。</p>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '120 days',
    200,
    1
),

-- 4. Docker Compose pitfalls
(
    'demo-docker-compose-deployment-pitfalls',
    'Docker Compose 部署个人项目的几个坑',
    '把一个小项目用 docker-compose 部到自己的 VPS 上,看起来五分钟搞定,实际上经常半天填坑。',
    E'## 写在前面\n\n把一个小项目用 docker-compose 部到自己的 VPS 上,看起来五分钟搞定,实际上经常半天填坑。\n\n## 坑 1: 数据库数据没持久化\n\n```yaml\nservices:\n  db:\n    image: postgres:16\n    # 没有 volumes —— 重启数据全没\n```\n\n一定要挂 named volume,不要用匿名卷。\n\n## 坑 2: 服务启动顺序\n\n`depends_on` 只保证启动顺序,不保证服务可用。Postgres 进程起来不代表能接受连接。要么后端加重试,要么用 `healthcheck` + `depends_on.condition: service_healthy`。\n\n## 坑 3: 把 .env 提交到 git\n\n`.gitignore` 里 `.env`,提供一份 `.env.example`。这条没什么技术含量,但出过事的人都记得很牢。\n\n## 一句话总结\n\n部署的 80% 工作是"让它在出错时还能恢复",而不是"让它跑起来"。\n',
    '<h2>写在前面</h2><p>把一个小项目用 docker-compose 部到自己的 VPS 上,看起来五分钟搞定,实际上经常半天填坑。</p><h2>坑 1: 数据库数据没持久化</h2><pre><code class="language-yaml">services:\n  db:\n    image: postgres:16\n    # 没有 volumes —— 重启数据全没</code></pre><p>一定要挂 named volume,不要用匿名卷。</p><h2>坑 2: 服务启动顺序</h2><p><code>depends_on</code> 只保证启动顺序,不保证服务可用。Postgres 进程起来不代表能接受连接。要么后端加重试,要么用 <code>healthcheck</code> + <code>depends_on.condition: service_healthy</code>。</p><h2>坑 3: 把 .env 提交到 git</h2><p><code>.gitignore</code> 里 <code>.env</code>,提供一份 <code>.env.example</code>。这条没什么技术含量,但出过事的人都记得很牢。</p><h2>一句话总结</h2><p>部署的 80% 工作是"让它在出错时还能恢复",而不是"让它跑起来"。</p>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '100 days',
    220,
    1
),

-- 5. Book notes: DDIA
(
    'demo-book-notes-designing-data-intensive-applications',
    '读书笔记:《Designing Data-Intensive Applications》',
    '一本"反碎片化"的书。同样讲分布式系统,网上博客是一堆点,这本书把它们连成一张图。',
    E'## 总评\n\n一本"反碎片化"的书。同样讲分布式系统,网上博客是一堆点,这本书把它们连成一张图。\n\n## 我最有共鸣的几章\n\n- **第 5 章 复制**:Leader-follower 看似简单,但故障切换是无穷的细节\n- **第 7 章 事务**:隔离级别不是越高越好,大多数业务在 Read Committed 下就够用\n- **第 9 章 一致性与共识**:Linearizability、Causal、Sequential 几个一致性模型的对比清晰得像教科书\n\n## 一个改变我看法的观点\n\n> "Eventual consistency" 这个词太弱了,它没告诉你"最终"是多久。\n\n作者更喜欢"convergent"——强调"会收敛"而不是"会延迟"。\n\n## 行动项\n\n- 写存储相关的代码前先想一遍:这次操作的一致性诉求到底有多强?\n- 不要默认上分布式事务,先看能不能用幂等 + 重试解决。\n',
    '<h2>总评</h2><p>一本"反碎片化"的书。同样讲分布式系统,网上博客是一堆点,这本书把它们连成一张图。</p><h2>我最有共鸣的几章</h2><ul><li><strong>第 5 章 复制</strong>:Leader-follower 看似简单,但故障切换是无穷的细节</li><li><strong>第 7 章 事务</strong>:隔离级别不是越高越好,大多数业务在 Read Committed 下就够用</li><li><strong>第 9 章 一致性与共识</strong>:Linearizability、Causal、Sequential 几个一致性模型的对比清晰得像教科书</li></ul><h2>一个改变我看法的观点</h2><blockquote><p>"Eventual consistency" 这个词太弱了,它没告诉你"最终"是多久。</p></blockquote><p>作者更喜欢"convergent"——强调"会收敛"而不是"会延迟"。</p><h2>行动项</h2><ul><li>写存储相关的代码前先想一遍:这次操作的一致性诉求到底有多强?</li><li>不要默认上分布式事务,先看能不能用幂等 + 重试解决。</li></ul>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '80 days',
    240,
    1
),

-- 6. Weekend life post
(
    'demo-weekend-coffee-machine-repair',
    '周末记:把咖啡机拆了又装回去',
    '周六醒来发现咖啡机不出水了。按提示除垢、清洗,折腾两小时还是不行。索性把外壳拧开,看清楚水路怎么走的。',
    E'周六醒来发现咖啡机不出水了。\n\n按提示除垢、清洗,折腾两小时还是不行。索性把外壳拧开,看清楚水路怎么走的——结果发现是一根硅胶管被水垢顶变形了。\n\n剪掉变形的一截,接回去,咖啡机又活了。\n\n修家电这种事,一半是手艺,一半是耐心。每次都告诉自己下次再坏直接换,然后下次又把它拆开。\n\n下午去附近书店翻了一会儿书,买了本不打算立刻读的小说。回家路上买了菜,做了个味噌汤。\n\n普通的一天,但比加班舒服一万倍。\n',
    '<p>周六醒来发现咖啡机不出水了。</p><p>按提示除垢、清洗,折腾两小时还是不行。索性把外壳拧开,看清楚水路怎么走的——结果发现是一根硅胶管被水垢顶变形了。</p><p>剪掉变形的一截,接回去,咖啡机又活了。</p><p>修家电这种事,一半是手艺,一半是耐心。每次都告诉自己下次再坏直接换,然后下次又把它拆开。</p><p>下午去附近书店翻了一会儿书,买了本不打算立刻读的小说。回家路上买了菜,做了个味噌汤。</p><p>普通的一天,但比加班舒服一万倍。</p>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '60 days',
    120,
    1
),

-- 7. Travel: Kyoto
(
    'demo-travel-kyoto-tips',
    '去了一趟京都:那些攻略不会写的细节',
    '地铁卡(ICOCA)直接在便利店充值就行,不用专门去车站。但如果是从关西机场进京都,直接买"HARUKA + ICOCA 套票"更划算。',
    E'## 关于交通\n\n地铁卡(ICOCA)直接在便利店充值就行,不用专门去车站。但如果是从关西机场进京都,直接买"HARUKA + ICOCA 套票"更划算。\n\n## 关于吃\n\n- 拉面在京都不是强项。想吃面的话,关西的乌冬反而更地道。\n- 锦市场是观光价,本地人不去。\n- 真正好吃的家常店往往没招牌,Google Maps 上评论很少但分高的那种就对了。\n\n## 关于寺院\n\n- 早上 8 点到清水寺,跟下午 3 点是两个世界。\n- 苔寺需要提前预约,而且要抄经。值得去一次。\n\n## 关于钱\n\n现金还是要带的。小店、地藏王、神社的赛钱箱都吃硬币。\n',
    '<h2>关于交通</h2><p>地铁卡(ICOCA)直接在便利店充值就行,不用专门去车站。但如果是从关西机场进京都,直接买"HARUKA + ICOCA 套票"更划算。</p><h2>关于吃</h2><ul><li>拉面在京都不是强项。想吃面的话,关西的乌冬反而更地道。</li><li>锦市场是观光价,本地人不去。</li><li>真正好吃的家常店往往没招牌,Google Maps 上评论很少但分高的那种就对了。</li></ul><h2>关于寺院</h2><ul><li>早上 8 点到清水寺,跟下午 3 点是两个世界。</li><li>苔寺需要提前预约,而且要抄经。值得去一次。</li></ul><h2>关于钱</h2><p>现金还是要带的。小店、地藏王、神社的赛钱箱都吃硬币。</p>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '45 days',
    190,
    1
),

-- 8. Machine learning intro
(
    'demo-machine-learning-andrew-ng-course',
    '机器学习入门:为什么 Andrew Ng 的课依然值得看',
    '虽然 2024 年才入门 ML 听起来有点晚,但 Andrew Ng 的 Coursera 课依然是我推荐的第一门。',
    E'## 背景\n\n虽然 2024 年才入门 ML 听起来有点晚,但 Andrew Ng 的 Coursera 课依然是我推荐的第一门。\n\n## 它好在哪\n\n- **不跳步骤**。线性回归的梯度下降一行一行推给你看。\n- **数学只在必要时出现**,不会一上来就甩你 100 页矩阵推导。\n- **作业实操**。手写一遍梯度下降比看十遍理论都管用。\n\n## 配合什么看\n\n- 看完课配合《Hands-On Machine Learning》(O''Reilly 那本)动手做项目\n- Kaggle 找一个 getting started 比赛,从头跑通一遍 baseline\n\n## 小心的几个误区\n\n1. 把 ML 等同于 deep learning。99% 的业务问题,XGBoost 就够了。\n2. 沉迷调参,不重视特征工程。\n3. 用 accuracy 作为唯一指标。类别不平衡时它毫无意义。\n',
    '<h2>背景</h2><p>虽然 2024 年才入门 ML 听起来有点晚,但 Andrew Ng 的 Coursera 课依然是我推荐的第一门。</p><h2>它好在哪</h2><ul><li><strong>不跳步骤</strong>。线性回归的梯度下降一行一行推给你看。</li><li><strong>数学只在必要时出现</strong>,不会一上来就甩你 100 页矩阵推导。</li><li><strong>作业实操</strong>。手写一遍梯度下降比看十遍理论都管用。</li></ul><h2>配合什么看</h2><ul><li>看完课配合《Hands-On Machine Learning》(O''Reilly 那本)动手做项目</li><li>Kaggle 找一个 getting started 比赛,从头跑通一遍 baseline</li></ul><h2>小心的几个误区</h2><ol><li>把 ML 等同于 deep learning。99% 的业务问题,XGBoost 就够了。</li><li>沉迷调参,不重视特征工程。</li><li>用 accuracy 作为唯一指标。类别不平衡时它毫无意义。</li></ol>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '25 days',
    260,
    1
),

-- 9. Security checklist
(
    'demo-small-project-security-audit-checklist',
    '给小项目做基本安全审计的 checklist',
    '不是给 enterprise 用的,是给"个人博客、副业项目、小工具"用的——你不会上专门的安全团队,但也不想哪天被人拿去挖矿。',
    E'## 谁需要这份 checklist\n\n不是给 enterprise 用的,是给"个人博客、副业项目、小工具"用的——你不会上专门的安全团队,但也不想哪天被人拿去挖矿。\n\n## Web 服务\n\n- [ ] HTTPS,不用 http\n- [ ] 关掉 server header 的版本号(Nginx 的 `server_tokens off`)\n- [ ] 限制 CORS 来源,不要 `*`\n- [ ] 至少一个简单的 rate limit(每 IP 每分钟 N 次)\n\n## 认证\n\n- [ ] 密码用 Argon2 或 bcrypt,不要 MD5/SHA1\n- [ ] JWT secret 长度 >= 32 字节\n- [ ] token 过期时间合理(24h 或更短)\n- [ ] 登录失败不区分"用户名不存在"和"密码错误"\n\n## 数据库\n\n- [ ] 业务账号不要用 superuser\n- [ ] 数据库端口不暴露公网\n- [ ] 用预编译参数,杜绝 SQL 注入(sqlx/sea-orm 默认就是)\n- [ ] 备份策略,且定期演练恢复\n\n## 服务器\n\n- [ ] SSH 关密码登录,只允许 key\n- [ ] fail2ban 装一下\n- [ ] 自动安全更新(unattended-upgrades)\n- [ ] 防火墙只开必要的端口\n\n## 一句话\n\n安全是个工程问题,不是一次性任务。每次部署前过一遍这个 checklist,90% 的低级错误就能避免。\n',
    '<h2>谁需要这份 checklist</h2><p>不是给 enterprise 用的,是给"个人博客、副业项目、小工具"用的——你不会上专门的安全团队,但也不想哪天被人拿去挖矿。</p><h2>Web 服务</h2><ul><li>HTTPS,不用 http</li><li>关掉 server header 的版本号(Nginx 的 <code>server_tokens off</code>)</li><li>限制 CORS 来源,不要 <code>*</code></li><li>至少一个简单的 rate limit(每 IP 每分钟 N 次)</li></ul><h2>认证</h2><ul><li>密码用 Argon2 或 bcrypt,不要 MD5/SHA1</li><li>JWT secret 长度 &gt;= 32 字节</li><li>token 过期时间合理(24h 或更短)</li><li>登录失败不区分"用户名不存在"和"密码错误"</li></ul><h2>数据库</h2><ul><li>业务账号不要用 superuser</li><li>数据库端口不暴露公网</li><li>用预编译参数,杜绝 SQL 注入(sqlx/sea-orm 默认就是)</li><li>备份策略,且定期演练恢复</li></ul><h2>服务器</h2><ul><li>SSH 关密码登录,只允许 key</li><li>fail2ban 装一下</li><li>自动安全更新(unattended-upgrades)</li><li>防火墙只开必要的端口</li></ul><h2>一句话</h2><p>安全是个工程问题,不是一次性任务。每次部署前过一遍这个 checklist,90% 的低级错误就能避免。</p>',
    'published',
    (SELECT id FROM users WHERE username = 'admin'),
    now() - INTERVAL '10 days',
    300,
    2
)

ON CONFLICT (slug) DO NOTHING;

-- ─── Post → tag associations ──────────────────────────────────────────────────

INSERT INTO post_tags (post_id, tag_id)
SELECT p.id, t.id
FROM (VALUES
    ('demo-rust-ownership-borrow-checker',                  'rust'),
    ('demo-vue3-composition-api-antipatterns',              'vue'),
    ('demo-vue3-composition-api-antipatterns',              'frontend'),
    ('demo-postgres-when-not-to-index',                     'postgres'),
    ('demo-postgres-when-not-to-index',                     'database'),
    ('demo-docker-compose-deployment-pitfalls',             'devops'),
    ('demo-docker-compose-deployment-pitfalls',             'docker'),
    ('demo-book-notes-designing-data-intensive-applications','reading'),
    ('demo-weekend-coffee-machine-repair',                  'life'),
    ('demo-travel-kyoto-tips',                              'travel'),
    ('demo-machine-learning-andrew-ng-course',              'ml'),
    ('demo-machine-learning-andrew-ng-course',              'python'),
    ('demo-small-project-security-audit-checklist',         'security')
) AS v(post_slug, tag_slug)
JOIN posts p ON p.slug = v.post_slug
JOIN tags  t ON t.slug = v.tag_slug
ON CONFLICT DO NOTHING;
