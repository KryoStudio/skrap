// Skrap's motion: what appears slides in from the left, slanted like the logo, and straightens;
// what goes flies off to the right. Durations come in already reduced when animations are off.
import { cubicIn, cubicOut, expoOut } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';

export function slideIn(
  _node: Element,
  { delay = 0, duration = 320, x = -28, skew = -12 }: { delay?: number; duration?: number; x?: number; skew?: number } = {},
): TransitionConfig {
  return {
    delay,
    duration,
    easing: expoOut,
    css: (t, u) => `transform: translateX(${u * x}px) skewX(${u * skew}deg); opacity: ${Math.min(1, t * 1.6)}`,
  };
}

export function flyOut(
  _node: Element,
  { delay = 0, duration = 220, x = 48 }: { delay?: number; duration?: number; x?: number } = {},
): TransitionConfig {
  return {
    delay,
    duration,
    easing: cubicIn,
    css: (t, u) => `transform: translateX(${u * x}px); opacity: ${t}`,
  };
}

export function fade(_node: Element, { delay = 0, duration = 200 }: { delay?: number; duration?: number } = {}): TransitionConfig {
  return { delay, duration, easing: cubicOut, css: (t) => `opacity: ${t}` };
}
