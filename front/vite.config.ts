import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const configuredBase =
	process.env.BASE_PATH ?? (process.env.NODE_ENV === 'production' ? '/Hackathon-SketchDate' : '');
if (configuredBase !== '' && !configuredBase.startsWith('/')) {
	throw new Error('BASE_PATH must be empty or start with a forward slash.');
}
const githubPagesBase = configuredBase as '' | `/${string}`;

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			paths: {
				base: githubPagesBase
			},
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			adapter: adapter({
				pages: 'build',
				assets: 'build',
				strict: true
			})
		})
	]
});
