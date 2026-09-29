import type { Preview } from '@storybook/react';

const preview: Preview = {
  parameters: {
    actions: { argTypesRegex: '^on[A-Z].*' },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    backgrounds: {
      default: 'space-dark',
      values: [
        { name: 'space-dark', value: '#0b1020' },
        { name: 'dark', value: '#1E293B' },
        { name: 'light', value: '#F8FAFC' },
      ],
    },
  },
};

export default preview;
