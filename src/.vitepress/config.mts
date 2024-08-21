import { defineConfig } from 'vitepress'

export default defineConfig({
  base: "/CEMCL/",
  title: "CE Minecraft Launcher",
  description: "CE Minecraft Launcher",
  lang: "zh-CN",
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
    },
    en: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/en' },
          { text: 'Download', link: '/en/download' },
          { text: 'Document', link: '/en/docs', activeMatch: '/en/docs/' },
          { text: 'About', link: '/en/about' },
        ],
        sidebar: {
          '/en/docs/': [
            {
              text: 'Document',
              items: [
                { text: 'Configuration Files', link: '/en/docs/config.md' },
              ]
            },
          ]
        },
      }
    },
  },
  themeConfig: {
    nav: [
      { text: '主页', link: '/' },
      { text: '下载', link: '/download' },
      { text: '文档', link: '/docs', activeMatch: '/docs/' },
      { text: '关于', link: '/about' },
    ],

    search: {
      provider: 'local'
    },

    sidebar: {
      '/docs/': [
        {
          text: '文档',
          items: [
            { text: '配置文件说明', link: '/docs/config.md' },
          ]
        },
      ]
    },
  }
})
