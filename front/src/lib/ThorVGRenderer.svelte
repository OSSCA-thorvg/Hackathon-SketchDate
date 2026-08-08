<script lang="ts">
	import ThorVG, {
		type Canvas as ThorVGCanvas,
		type Matrix,
		type Paint,
		type Picture,
		type Shape,
		type ThorVGNamespace
	} from '@thorvg/webcanvas';
	import thorvgWasmUrl from '../../node_modules/@thorvg/webcanvas/dist/thorvg.wasm?url';
	import { onMount } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';

	type PaintPath = {
		commands: readonly number[];
		points: ReadonlyArray<readonly number[]>;
	};

	type Props = {
		source: string;
		sourceKey?: string;
		paintTransforms?: Readonly<Record<string, Matrix>>;
		paintOpacities?: Readonly<Record<string, number>>;
		paintPaths?: Readonly<Record<string, PaintPath>>;
		width?: number;
		height?: number;
		label?: string;
		class?: string;
		onerror?: (error: Error) => void;
	};

	let {
		source,
		sourceKey,
		paintTransforms = {},
		paintOpacities = {},
		paintPaths = {},
		width = 800,
		height = 600,
		label = 'Vector character',
		class: className = '',
		onerror
	}: Props = $props();

	let canvasElement: HTMLCanvasElement;
	let tvg: ThorVGNamespace | null = null;
	let surface: ThorVGCanvas | null = null;
	let picture: Picture | null = null;
	let renderFrame: number | null = null;
	let latestSource = '';
	let latestSourceKey = '';
	let loadedSourceKey: string | null = null;
	let latestPaintTransforms: Readonly<Record<string, Matrix>> = {};
	let latestPaintOpacities: Readonly<Record<string, number>> = {};
	let latestPaintPaths: Readonly<Record<string, PaintPath>> = {};
	let latestWidth = 1;
	let latestHeight = 1;
	const paintCache = new SvelteMap<string, Paint | null>();

	function reportError(error: unknown): void {
		const normalized = error instanceof Error ? error : new Error(String(error));
		console.error('ThorVG renderer:', normalized);
		onerror?.(normalized);
	}

	function scheduleRender(): void {
		if (!surface || !tvg || renderFrame !== null) return;

		renderFrame = requestAnimationFrame(() => {
			renderFrame = null;
			drawLatestSource();
		});
	}

	function drawLatestSource(): void {
		if (!surface || !tvg) return;

		try {
			surface.resize(latestWidth, latestHeight);

			if (!picture || loadedSourceKey !== latestSourceKey) {
				const nextPicture = new tvg.Picture();
				nextPicture.accessible = true;
				nextPicture.load(latestSource, { type: 'svg' });
				nextPicture.size(latestWidth, latestHeight);

				const previousPicture = picture;
				if (previousPicture) surface.remove(previousPicture);
				surface.add(nextPicture);
				picture = nextPicture;
				loadedSourceKey = latestSourceKey;
				paintCache.clear();
				previousPicture?.dispose();
			}

			const getPaint = (name: string): Paint | null => {
				if (!picture) return null;
				if (!paintCache.has(name)) paintCache.set(name, picture.paint(name));
				return paintCache.get(name) ?? null;
			};

			for (const [name, matrix] of Object.entries(latestPaintTransforms)) {
				getPaint(name)?.transform(matrix);
			}

			for (const [name, opacity] of Object.entries(latestPaintOpacities)) {
				getPaint(name)?.opacity(Math.min(255, Math.max(0, Math.round(opacity))));
			}

			for (const [name, path] of Object.entries(latestPaintPaths)) {
				const paint = getPaint(name);
				if (paint instanceof tvg.Shape) {
					paint
						.reset()
						.appendPath(path.commands as Parameters<Shape['appendPath']>[0], path.points);
				}
			}

			// Reattach the retained picture so ThorVG invalidates its nested accessible paints.
			// The parsed SVG and paint objects remain alive; only the canvas scene reference changes.
			if (picture) surface.remove(picture).add(picture);
			surface.update().render();
		} catch (error) {
			reportError(error);
		}
	}

	$effect(() => {
		latestSource = source;
		latestSourceKey = sourceKey ?? source;
		latestPaintTransforms = paintTransforms;
		latestPaintOpacities = paintOpacities;
		latestPaintPaths = paintPaths;
		latestWidth = Math.max(1, Math.round(width));
		latestHeight = Math.max(1, Math.round(height));
		scheduleRender();
	});

	onMount(() => {
		let destroyed = false;
		const canvasId = `thorvg-${crypto.randomUUID()}`;
		canvasElement.id = canvasId;

		void ThorVG.init({
			renderer: 'gl',
			locateFile: (path) => (path.endsWith('.wasm') ? thorvgWasmUrl : path),
			onError: (error) => reportError(error)
		})
			.then((namespace) => {
				if (destroyed) return;

				tvg = namespace;
				surface = new namespace.Canvas(`#${canvasId}`, {
					width: latestWidth,
					height: latestHeight
				});
				scheduleRender();
			})
			.catch(reportError);

		return () => {
			destroyed = true;
			if (renderFrame !== null) cancelAnimationFrame(renderFrame);
			if (picture && surface) surface.remove(picture);
			picture?.dispose();
			surface?.destroy();
			picture = null;
			surface = null;
			tvg = null;
		};
	});
</script>

<div class={className}>
	<canvas bind:this={canvasElement} aria-label={label}></canvas>
</div>

<style>
	canvas {
		display: block;
		width: 100% !important;
		height: 100% !important;
	}
</style>
