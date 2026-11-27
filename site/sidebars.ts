import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docs: [
    'spec',
    'plan',
    {
      type: 'category',
      label: 'Design',
      collapsed: false,
      items: ['design/logo', 'design/terminal-ui', 'design/questionnaire'],
    },
    {
      type: 'link',
      label: 'Contributing',
      href: 'https://github.com/Raghav2012Code/nib/blob/main/AGENTS.md',
    },
  ],
};

export default sidebars;
