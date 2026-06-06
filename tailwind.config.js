/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "var(--theme-primary)",
          hover: "var(--theme-primary-hover)",
          foreground: "var(--theme-primary-foreground)",
        },
        accent: {
          DEFAULT: "var(--theme-accent)",
          foreground: "var(--theme-accent-foreground)",
        },
        ring: "var(--theme-ring)",
      }
    },
  },
  plugins: [],
}
