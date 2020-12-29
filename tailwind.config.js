const defaultTheme = require('tailwindcss/defaultTheme');
const colors = require('tailwindcss/colors');

module.exports = {
  purge: ['./src/**/*.html', './src/**/*.svelte'],
  plugins: [require('@tailwindcss/forms'), require('@tailwindcss/typography')],
  darkMode: 'class',
  theme: {
    colors: {
      primary: colors.teal,
      amber: colors.amber,
      gray: colors.gray,
      transparent: 'transparent',
      current: 'currentColor',
      white: 'white',
      black: 'black',
    },
    extend: {
      fontFamily: {
        sans: ['Inter', ...defaultTheme.fontFamily.sans],
        serif: ['Merriweather', ...defaultTheme.fontFamily.serif],
        mono: ['Inconsolata', ...defaultTheme.fontFamily.mono],
      },
    },
  },
  variants: {},
};
