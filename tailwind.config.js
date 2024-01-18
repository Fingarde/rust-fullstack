/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/view/**/*.{html,twig,tera}',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
  /*safelist: [
    {
      pattern: /./, // the "." means "everything"
    },
  ],*/
}

