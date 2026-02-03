import { defineConfig, HeadConfig} from 'vitepress'
import llmstxt from 'vitepress-plugin-llms'

const transformHead = ({ }): HeadConfig[] => {
  const head: HeadConfig[] = []
  head.push(['link', { rel: 'icon', type: 'image/svg+xml', href: '/yozefu/favicon.svg' }])
  return head
}


const description = 'Browse and query Kafka from the terminal.'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  vite: {
		plugins: [llmstxt()],
	},
  transformHead: transformHead,
  title: "Yōzefu",
  description: "A TUI to explore Kafka clusters and data",
  base: '/yozefu/',
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: 'https://maif.github.io/yozefu/favicon.svg' }],
    ['meta', { property: 'og:url', content: 'https://maif.github.io/yozefu/' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Yōzefu' }],
    ['meta', { property: 'og:description', content: description }],
    ['meta', { property: 'og:image', content: 'https://maif.github.io/yozefu/open-graph.png' }],
    ['meta', { name: 'twitter:card', content: 'https://maif.github.io/yozefu/open-graph.png' }],
    ['meta', { name: 'twitter:domain', content: 'maif.github.io' }],
    ['meta', { name: 'twitter:url', content: 'https://maif.github.io/yozefu/' }],
    ['meta', { name: 'twitter:title', content: 'Yōzefu' }],
    ['meta', { name: 'twitter:description', content: description }],
    ['meta', { name: 'twitter:image', content: 'https://maif.github.io/yozefu/open-graph.png' }],
  ],
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
      <a target="_self" rel="noopener noreferrer" href="https://maif.github.io/"><img width="500" height="392" class="footer-maif" alt="MAIF logo" src="https://maif.github.io/yozefu/maif.svg" /></a><br /><a target="_self" rel="noopener noreferrer" href="https://maif.github.io/">OSS by MAIF</a>, released under Apache License, Version 2.0`,
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
