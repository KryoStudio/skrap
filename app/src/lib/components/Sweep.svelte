<!-- The scan's signature: a slanted acid line sweeps the screen from left to right like a
     squeegee, and the specks of junk it meets fly off to the right, lit up. Canvas 2D, one frame
     per display refresh, nothing when animations are reduced. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { motion } from '$lib/settings.svelte';

  let canvas: HTMLCanvasElement;

  const TAN = Math.tan((12 * Math.PI) / 180);
  const PASS_MS = 1500;
  const SPECKS = 140;
  const ACID = '214, 255, 61';

  interface Speck {
    x: number;
    y: number;
    r: number;
    vx: number;
    life: number;
    hit: boolean;
  }

  onMount(() => {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    let width = 0;
    let height = 0;
    let specks: Speck[] = [];
    let frame = 0;
    let passStart = performance.now();
    let last = passStart;

    const scatter = () => {
      specks = Array.from({ length: SPECKS }, () => ({
        x: Math.random() * width,
        y: Math.random() * height,
        r: 0.8 + Math.random() * 1.6,
        vx: 0,
        life: 1,
        hit: false,
      }));
    };

    const resize = () => {
      const dpr = window.devicePixelRatio || 1;
      width = canvas.clientWidth;
      height = canvas.clientHeight;
      canvas.width = Math.round(width * dpr);
      canvas.height = Math.round(height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      scatter();
    };

    /** The line's x at the bottom of the screen (it leans right going up, like the logo). */
    const lineAt = (progress: number) => {
      const lean = height * TAN;
      return -lean - 40 + progress * (width + lean + 80);
    };

    /** Fast in the middle, gentle at both ends. */
    const ease = (p: number) => (p < 0.5 ? 4 * p ** 3 : 1 - (-2 * p + 2) ** 3 / 2);

    /** Draws the sweep at `still` progress (animations off), or where the time says. */
    const draw = (now: number, still?: number) => {
      const dt = Math.min(48, now - last);
      last = now;
      let progress = still ?? (now - passStart) / PASS_MS;
      if (progress >= 1) {
        passStart = now;
        progress = 0;
        scatter();
      }
      const x = lineAt(ease(progress));
      const lean = height * TAN;

      ctx.clearRect(0, 0, width, height);

      // The wake: a slanted acid gradient behind the line.
      const wake = ctx.createLinearGradient(x - 320, 0, x + lean, 0);
      wake.addColorStop(0, `rgba(${ACID}, 0)`);
      wake.addColorStop(1, `rgba(${ACID}, 0.10)`);
      ctx.fillStyle = wake;
      ctx.beginPath();
      ctx.moveTo(x - 320, height);
      ctx.lineTo(x, height);
      ctx.lineTo(x + lean, 0);
      ctx.lineTo(x - 320 + lean, 0);
      ctx.closePath();
      ctx.fill();

      // The specks: grey until the line reaches them, then acid, flying right and fading.
      for (const s of specks) {
        const edge = x + (height - s.y) * TAN;
        if (!s.hit && s.x <= edge) {
          s.hit = true;
          s.vx = 0.4 + Math.random() * 0.9;
        }
        if (s.hit) {
          s.x += s.vx * dt;
          s.life -= dt / 700;
          if (s.life <= 0) continue;
          ctx.fillStyle = `rgba(${ACID}, ${0.85 * s.life})`;
          ctx.fillRect(s.x - s.vx * 18, s.y - s.r / 2, s.vx * 18, s.r);
        } else {
          ctx.fillStyle = 'rgba(140, 140, 147, 0.35)';
          ctx.fillRect(s.x, s.y, s.r, s.r);
        }
      }

      // The line itself, with a glow.
      ctx.save();
      ctx.shadowColor = `rgba(${ACID}, 0.8)`;
      ctx.shadowBlur = 28;
      ctx.fillStyle = `rgb(${ACID})`;
      ctx.beginPath();
      ctx.moveTo(x - 3, height);
      ctx.lineTo(x + 3, height);
      ctx.lineTo(x + 3 + lean, 0);
      ctx.lineTo(x - 3 + lean, 0);
      ctx.closePath();
      ctx.fill();
      ctx.restore();

      if (still === undefined) frame = requestAnimationFrame((t) => draw(t));
    };

    /** Animations off: one still frame, the line two thirds across, what it passed already gone. */
    const still = () => {
      const x = lineAt(ease(0.62));
      specks = specks.filter((s) => s.x > x + (height - s.y) * TAN);
      draw(performance.now(), 0.62);
    };

    resize();
    const observer = new ResizeObserver(() => {
      resize();
      if (motion.reduced) still();
    });
    observer.observe(canvas);
    if (motion.reduced) still();
    else frame = requestAnimationFrame((t) => draw(t));
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  });
</script>

<canvas bind:this={canvas} aria-hidden="true"></canvas>

<style>
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
</style>
