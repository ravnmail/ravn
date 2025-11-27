import { cva, type VariantProps } from 'class-variance-authority'

export { default as Badge } from './Badge.vue'
export { default as SplitBadge } from './SplitBadge.vue'

export const badgeVariants = cva(
  'inline-flex items-center rounded-md border font-semibold focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        // default: 'border-transparent bg-primary text-primary-foreground shadow',
        default: 'border-transparent bg-badge-secondary-background text-badge-secondary-foreground',
        ai: 'border-transparent bg-badge-ai-background text-badge-ai-foreground',
        destructive: 'border-transparent bg-badge-destructive-background text-badge-destructive-foreground',
        accent: 'border-transparent bg-badge-accent-background text-badge-accent-foreground',
        warning: 'border-transparent bg-badge-warning-background text-badge-warning-foreground',
        info: 'border-transparent bg-badge-info-background text-badge-info-foreground',
        success: 'border-transparent bg-badge-success-background text-badge-success-foreground',
        outline: 'border-badge-outline-border text-badge-outline-foreground',
        surface: 'border-transparent bg-badge-surface-background text-badge-surface-foreground',
        secondary: 'border-transparent bg-badge-secondary-background text-badge-secondary-foreground',
        primary: 'border-transparent bg-badge-primary-background text-badge-primary-foreground',
      },
      size: {
        sm: 'text-xs px-2 py-0.5',
        default: 'text-sm px-2.5 py-0.5',
        lg: 'text-lg px-2.5 py-0.5',
      }
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export type BadgeVariants = VariantProps<typeof badgeVariants>
