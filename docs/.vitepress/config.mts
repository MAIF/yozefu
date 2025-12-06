import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  
  title: "Yōzefu",
  description: "Interactive TUI to explore Kafka clusters and data",
  base: '/yozefu/',
  themeConfig: {
    search: {
      provider: 'local'
    },
    nav: [
      { text: 'Documentation', link: '/getting-started/' },
      { text: 'docs.rs', link: 'https://docs.rs/crate/yozefu/latest' },
      { text: 'crates.io', link: 'https://crates.io/crates/yozefu' }
    ],
     footer: {
      message: `
      <a href="https://maif.github.io/"><img class="footer-maif" src="https://maif.github.io/yozefu/maif.svg" /></a><br /><a href="https://maif.github.io/">OSS by MAIF</a>, released under Apache License, Version 2.0</p>`,
    },
    sidebar: [
      {
        text: 'Introduction',
        items: [
          { text: 'What is Yōzefu?', link: '/what-is-yozefu/' },
          { text: 'Getting Started', link: '/getting-started/' },
          { text: 'Keybindings', link: '/keybindings/' },
          
        ]
      },
      {
        text: 'Configuration',
        items: [
          { text: 'General', link: '/configuration/' },
          { text: 'TLS', link: '/tls/' },
          { text: 'Schema Registry', link: '/schema-registry/' },
          { text: 'Themes', link: '/themes/' },
          { text: 'URL Templates', link: '/url-templates/' },
        ]
      },
      {
        text: 'Search',
        items: [

          { text: 'Creating a search filter', link: '/search-filter/' },
          { text: 'Examples', link: '/query-language/' },
        ]
      },
      {
        text: 'Internals',
        items: [
          { text: 'JSON schemas', link: '/json-schemas/' },
        ]
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/MAIF/yozefu' }
    ]
  }
})
