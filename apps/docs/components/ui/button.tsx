import { cva, type VariantProps } from "class-variance-authority";
import { clsx, type ClassValue } from "clsx";
import type { ButtonHTMLAttributes } from "react";
import { twMerge } from "tailwind-merge";

function cn(...values: ClassValue[]) {
  return twMerge(clsx(values));
}

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-full text-sm font-semibold transition disabled:pointer-events-none disabled:opacity-45 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--omena-ring)] focus-visible:ring-offset-2",
  {
    variants: {
      variant: {
        primary:
          "bg-[var(--omena-ink)] px-5 py-2.5 text-[var(--omena-paper)] shadow-[0_8px_24px_rgba(35,29,24,0.18)] hover:-translate-y-0.5",
        secondary:
          "border border-[var(--omena-line)] bg-white/70 px-5 py-2.5 text-[var(--omena-ink)] hover:bg-white",
        ghost: "px-3 py-2 text-[var(--omena-muted)] hover:bg-black/5 hover:text-[var(--omena-ink)]",
      },
    },
    defaultVariants: {
      variant: "primary",
    },
  },
);

export function Button({
  className,
  variant,
  type = "button",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & VariantProps<typeof buttonVariants>) {
  return <button type={type} className={cn(buttonVariants({ variant }), className)} {...props} />;
}
