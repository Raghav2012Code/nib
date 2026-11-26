import {themes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'nib',
  tagline: 'Offline pastebin for your terminal.',
  favicon: 'img/nib-appicon.svg',

  future: {
    v4: true,
  },

  url: 'https://raghav2012code.github.io',
  baseUrl: '/nib/',
  organizationName: 'Raghav2012Code',
  projectName: 'nib',
  trailingSlash: false,

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          // /docs is the source. The site includes it by path and copies
          // nothing, so the spec cannot drift from here.
          path: '../docs',
          routeBasePath: 'docs',
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/Raghav2012Code/nib/edit/main/docs/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themes: [
    [
      require.resolve('@easyops-cn/docusaurus-search-local'),
      {
        hashed: true,
        indexDocs: true,
        indexBlog: false,
        docsRouteBasePath: 'docs',
        docsDir: '../docs',
        highlightSearchTermsOnTargetPage: true,
      },
    ],
  ],

  themeConfig: {
    image: 'img/nib-appicon.svg',
    colorMode: {
      defaultMode: 'dark',
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'nib',
      logo: {
        alt: 'nib',
        src: 'img/nib-appicon.svg',
      },
      items: [
        {type: 'docSidebar', sidebarId: 'docs', position: 'left', label: 'Docs'},
        {to: '/docs/spec', position: 'left', label: 'Spec'},
        {
          href: 'https://github.com/Raghav2012Code/nib',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'The spec', to: '/docs/spec'},
            {label: 'The plan', to: '/docs/plan'},
            {label: 'The mark', to: '/docs/design/logo'},
          ],
        },
        {
          title: 'Source',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/Raghav2012Code/nib',
            },
            {
              label: 'Issues',
              href: 'https://github.com/Raghav2012Code/nib/issues',
            },
          ],
        },
      ],
      copyright: 'MIT. Text lives here; nothing leaves the machine.',
    },
    prism: {
      theme: themes.vsLight,
      darkTheme: themes.vsDark,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
