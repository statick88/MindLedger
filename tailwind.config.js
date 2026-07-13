/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        /* MindLedger direct brand colors — for explicit use */
        teal: {
          50: '#e6f3f5',
          100: '#b3dce3',
          200: '#80c5d1',
          300: '#4daebe',
          400: '#269eaf',
          500: '#0F4C5C',
          600: '#0d4352',
          700: '#0a3543',
          800: '#072833',
          900: '#041a23',
        },
        sage: {
          50: '#f4f9f8',
          100: '#E5F1EE',
          200: '#cce5df',
          300: '#b2d9d0',
          400: '#99cdc1',
          500: '#80c1b2',
          600: '#66b5a3',
          700: '#4da994',
          800: '#339d85',
          900: '#1a9176',
        },
        coral: {
          50: '#fdf0ef',
          100: '#f9d4d2',
          200: '#f4b8b5',
          300: '#ef9c98',
          400: '#ea807b',
          500: '#E3645F',
          600: '#cc5a55',
          700: '#b24f4b',
          800: '#994441',
          900: '#7f3937',
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
      },
    },
  },
  plugins: [],
}
