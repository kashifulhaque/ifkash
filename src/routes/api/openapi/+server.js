import { json } from '@sveltejs/kit';
import openapi from '../../../../static/openapi.json' assert { type: 'json' };

export function GET() {
	return json(openapi, {
		headers: {
			'Cache-Control': 'public, max-age=3600'
		}
	});
}