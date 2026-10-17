/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "#0B1019",
        sidebar: "#101722",
        surface: "#151E2D",
        "surface-elevated": "#1C2839",
        "surface-hover": "#25354D",
        "surface-border": "#223047",
        "surface-border-subtle": "rgba(255, 255, 255, 0.06)",
        primary: {
          DEFAULT: "#39BDF8",
          hover: "#0EA5E9",
          dark: "#0284C7",
          muted: "rgba(57, 189, 248, 0.12)",
        },
        success: {
          DEFAULT: "#22C997",
          hover: "#1CB384",
          muted: "rgba(34, 201, 151, 0.12)",
        },
        warning: {
          DEFAULT: "#F4B544",
          hover: "#E0A130",
          muted: "rgba(244, 181, 68, 0.12)",
        },
        critical: {
          DEFAULT: "#F26778",
          hover: "#DE4E60",
          muted: "rgba(242, 103, 120, 0.12)",
        },
        accent: {
          cyan: "#38BDF8",
          emerald: "#22C997",
          amber: "#F4B544",
          rose: "#F26778",
          purple: "#A855F7",
        },
        slate: {
          850: "#131B2E",
          950: "#080C14",
        }
      },
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          '"Segoe UI"',
          "Inter",
          "Roboto",
          "Helvetica Neue",
          "sans-serif",
        ],
        mono: [
          '"JetBrains Mono"',
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Monaco",
          "Consolas",
          "monospace",
        ],
      },
      keyframes: {
        pulseSlow: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.4" },
        },
        shimmer: {
          "100%": { transform: "translateX(100%)" },
        },
      },
      animation: {
        "pulse-slow": "pulseSlow 2.5s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        shimmer: "shimmer 1.5s infinite",
      },
    },
  },
  plugins: [],
}
