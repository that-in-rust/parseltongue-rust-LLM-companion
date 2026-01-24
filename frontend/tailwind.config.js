/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Change type colors matching spec
        'change-added': '#22c55e',    // green-500
        'change-removed': '#ef4444',  // red-500
        'change-modified': '#f59e0b', // amber-500
        'change-affected': '#3b82f6', // blue-500
        'change-unchanged': '#6b7280', // gray-500
      },
    },
  },
  plugins: [],
};
