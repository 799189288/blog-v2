<script setup lang="ts">
// Homepage hero card. The copy here is intentionally a placeholder
// the blog owner can edit directly — name, bio, social links. Avatar
// is a CSS-drawn circle with the first initial so there's no broken
// image link until a real avatar URL goes in.

import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// Edit these four lines to personalize the hero.
const authorName = 'Leo'
const authorInitial = 'L'
const tagline = "Notes on building things — Rust, TypeScript, and the occasional rant."
const socials = [
  { label: 'GitHub', href: 'https://github.com/' },
  { label: 'Email', href: 'mailto:hello@example.com' },
  { label: 'RSS', href: '/rss.xml' },
]
</script>

<template>
  <header class="hero" :aria-label="t('home.aboutAria')">
    <div class="avatar" aria-hidden="true">{{ authorInitial }}</div>
    <div class="body">
      <h1 class="name">{{ authorName }}</h1>
      <p class="tagline">{{ tagline }}</p>
      <nav class="socials" :aria-label="t('home.socialsAria')">
        <a
          v-for="s in socials"
          :key="s.label"
          :href="s.href"
          class="social-link"
          rel="me noopener"
        >{{ s.label }}</a>
      </nav>
    </div>
  </header>
</template>

<style scoped>
.hero {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 24px 0 28px;
  margin-bottom: 24px;
  border-bottom: 1px solid var(--n-border-color, rgba(127, 127, 127, 0.15));
}
.avatar {
  flex: 0 0 auto;
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: var(--brand-color, #c0392b);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'EB Garamond', 'Noto Serif SC', Georgia, serif;
  font-weight: 600;
  font-size: 36px;
  user-select: none;
  letter-spacing: -0.02em;
}
.body {
  min-width: 0;
  flex: 1;
}
.name {
  margin: 0 0 6px;
  font-size: 28px;
  line-height: 1.2;
}
.tagline {
  margin: 0 0 10px;
  font-size: 14px;
  opacity: 0.75;
  line-height: 1.55;
}
.socials {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}
.social-link {
  font-size: 13px;
  text-decoration: none;
  color: inherit;
  opacity: 0.75;
  border-bottom: 1px solid currentColor;
  padding-bottom: 1px;
  transition: opacity 0.15s, color 0.15s;
}
.social-link:hover {
  opacity: 1;
  color: var(--brand-color, #c0392b);
}

@media (max-width: 480px) {
  .hero {
    flex-direction: column;
    align-items: flex-start;
    gap: 14px;
    padding: 18px 0 22px;
  }
  .avatar {
    width: 64px;
    height: 64px;
    font-size: 28px;
  }
  .name { font-size: 24px; }
}
</style>
