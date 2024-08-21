import { defineConfig } from 'vitepress'

export default defineConfig({
  base: "/CEMCL/",
  title: "CE Minecraft Launcher",
  description: "CE Minecraft Launcher",
  lang: "zh-CN",
  themeConfig: {
    nav: [
      { text: '主页', link: '/' },
      { text: '下载', link: '/download' },
      { text: '文档', link: '/docs', activeMatch: '/docs/' },
      { text: '关于', link: '/about' }
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
            { text: '配置文件说明(English)', link: '/docs/config_en.md' },
          ]
        },
      ]
    },
  }
})
