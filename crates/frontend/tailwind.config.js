/** @type {import('tailwindcss').Config} */
module.exports = {
  // Tailwind scans these files for class names to include in the output CSS.
  // Leptos class names live in .rs files (inside view! macros).
  content: [
    "./src/**/*.rs",
    "./index.html",
    "../../crates/shared/src/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        blush:     "#f4c2c2",
        sage:      "#8fae88",
        champagne: "#f7e7ce",
        ivory:     "#fffff0",
        charcoal:  "#36454f",
        gold:      "#c8a951",
        earth:     "#8b6f47",
        cream:     "#fdf8f0",
      },
      fontFamily: {
        serif:  ["Playfair Display", "Georgia", "Times New Roman", "serif"],
        sans:   ["Inter", "system-ui", "-apple-system", "sans-serif"],
        script: ["Dancing Script", "cursive"],
      },
    },
  },
  plugins: [],
};
