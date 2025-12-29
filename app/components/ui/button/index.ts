import { cva, type VariantProps } from 'class-variance-authority'

export { default as Button } from './Button.vue'

export const buttonVariants = cva(
  'cursor-pointer inline-flex items-center justify-center gap-2 whitespace-nowrap font-semibold transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'bg-button-secondary-background text-button-secondary-foreground shadow-sm hover:bg-button-secondary-background/80',
        primary: 'bg-button-primary-background text-button-primary-foreground shadow hover:bg-button-primary-hover/80',
        ai: 'bg-button-ai-background text-button-ai-foreground shadow hover:bg-button-ai-background/80',
        accent: 'bg-button-accent-background text-button-accent-foreground shadow-sm hover:bg-button-accent-hover shadow-lg shadow-button-accent-background/20',
        destructive: 'bg-button-destructive-background/20 text-button-destructive-foreground shadow-sm hover:bg-button-destructive-background/80',
        warning: 'bg-button-warning-background/20 text-button-warning-foreground shadow-sm hover:bg-button-warning-background/80',
        outline: 'text-button-outline-foreground border border-button-outline-border bg-transparent shadow-sm hover:bg-button-outline-hover/80',
        ghost: 'hover:bg-button-ghost-hover/80 hover:text-button-ghost-foreground',
        link: 'text-button-link-foreground underline-offset-4 hover:underline',
      },
      size: {
        default: 'px-4 py-1.5 rounded-lg',
        bar: 'h-7 rounded-md px-1.5',
        none: 'rounded-sm text-sm p-1',
        '2xs': 'h-5 rounded-sm px-1 text-xs',
        xs: 'h-7 rounded-md px-2 text-xs',
        sm: 'h-8 rounded-md px-3 text-sm',
        lg: 'h-10 rounded-lg px-8',
        icon: 'h-9 w-9 rounded-lg',
        rounded: 'rounded-full px-2',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'sm',
    },
  },
)

export type ButtonVariants = VariantProps<typeof buttonVariants>
