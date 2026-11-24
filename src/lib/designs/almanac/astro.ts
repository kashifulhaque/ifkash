/**
 * Calendar and sky helpers for the Almanac masthead. Everything here is a
 * pure function of a date, so there is no network call. Results are
 * approximations meant for a dateline, not for navigation.
 */

/** Bengaluru, in degrees. East longitude is positive. */
export const LAT = 12.97;
export const LON = 77.59;
/** India Standard Time offset, in minutes. */
const IST = 330;

const SYNODIC = 29.530588853;
/** A reference new moon: 2000-01-06 18:14 UTC. */
const NEW_MOON = Date.UTC(2000, 0, 6, 18, 14);

export interface Ymd {
	y: number;
	m: number;
	d: number;
}

/** The calendar date in Bengaluru for an instant. */
export function bangaloreDate(now: Date): Ymd {
	const parts = new Intl.DateTimeFormat('en-CA', {
		timeZone: 'Asia/Kolkata',
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
	}).formatToParts(now);
	const get = (t: string) => Number(parts.find((p) => p.type === t)?.value);
	return { y: get('year'), m: get('month'), d: get('day') };
}

export function dayOfYear({ y, m, d }: Ymd): number {
	return Math.round((Date.UTC(y, m - 1, d) - Date.UTC(y, 0, 0)) / 86_400_000);
}

export function isoWeek({ y, m, d }: Ymd): number {
	const date = new Date(Date.UTC(y, m - 1, d));
	const day = date.getUTCDay() || 7;
	date.setUTCDate(date.getUTCDate() + 4 - day);
	const yearStart = Date.UTC(date.getUTCFullYear(), 0, 1);
	return Math.ceil(((date.getTime() - yearStart) / 86_400_000 + 1) / 7);
}

export interface Moon {
	/** Position in the cycle, 0 (new) to 1. */
	fraction: number;
	/** Fraction of the disc that is lit, 0 to 1. */
	lit: number;
	name: string;
}

export function moonPhase(now: Date): Moon {
	const days = (now.getTime() - NEW_MOON) / 86_400_000;
	const fraction = (((days / SYNODIC) % 1) + 1) % 1;
	const lit = (1 - Math.cos(2 * Math.PI * fraction)) / 2;
	const names = [
		'New moon',
		'Waxing crescent',
		'First quarter',
		'Waxing gibbous',
		'Full moon',
		'Waning gibbous',
		'Last quarter',
		'Waning crescent',
	];
	return { fraction, lit, name: names[Math.round(fraction * 8) % 8] };
}

/**
 * SVG path for the lit part of the moon, in a disc of radius `r` centered
 * on the origin. Waning phases are mirrored by the caller.
 */
export function moonPath(fraction: number, r: number): { d: string; mirror: boolean } {
	const mirror = fraction >= 0.5;
	const f = mirror ? 1 - fraction : fraction;
	const c = Math.cos(2 * Math.PI * f);
	const rx = Math.abs(c) * r;
	const sweep = c > 0 ? 0 : 1;
	const d = `M0 ${-r} A${r} ${r} 0 0 1 0 ${r} A${rx.toFixed(2)} ${r} 0 0 ${sweep} 0 ${-r}Z`;
	return { d, mirror };
}

/** Local `HH:MM` for minutes after midnight IST. */
function hhmm(minutes: number): string {
	const m = ((Math.round(minutes) % 1440) + 1440) % 1440;
	return `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`;
}

/** Approximate sunrise and sunset in Bengaluru, as IST `HH:MM`. */
export function sunTimes(ymd: Ymd): { rise: string; set: string } {
	const n = dayOfYear(ymd);
	const g = ((2 * Math.PI) / 365) * (n - 1);
	const eqTime =
		229.18 *
		(0.000075 +
			0.001868 * Math.cos(g) -
			0.032077 * Math.sin(g) -
			0.014615 * Math.cos(2 * g) -
			0.040849 * Math.sin(2 * g));
	const decl =
		0.006918 -
		0.399912 * Math.cos(g) +
		0.070257 * Math.sin(g) -
		0.006758 * Math.cos(2 * g) +
		0.000907 * Math.sin(2 * g) -
		0.002697 * Math.cos(3 * g) +
		0.00148 * Math.sin(3 * g);
	const lat = (LAT * Math.PI) / 180;
	const cosHa =
		Math.cos((90.833 * Math.PI) / 180) / (Math.cos(lat) * Math.cos(decl)) -
		Math.tan(lat) * Math.tan(decl);
	const ha = (Math.acos(Math.max(-1, Math.min(1, cosHa))) * 180) / Math.PI;
	const riseUtc = 720 - 4 * (LON + ha) - eqTime;
	const setUtc = 720 - 4 * (LON - ha) - eqTime;
	return { rise: hhmm(riseUtc + IST), set: hhmm(setUtc + IST) };
}

const WEEKDAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const MONTHS = [
	'January',
	'February',
	'March',
	'April',
	'May',
	'June',
	'July',
	'August',
	'September',
	'October',
	'November',
	'December',
];

export interface Dateline {
	weekday: string;
	date: string;
	doy: number;
	week: number;
	moon: Moon;
	rise: string;
	set: string;
}

export function dateline(now = new Date()): Dateline {
	const ymd = bangaloreDate(now);
	const wd = new Date(Date.UTC(ymd.y, ymd.m - 1, ymd.d)).getUTCDay();
	return {
		weekday: WEEKDAYS[wd],
		date: `${ymd.d} ${MONTHS[ymd.m - 1]} ${ymd.y}`,
		doy: dayOfYear(ymd),
		week: isoWeek(ymd),
		moon: moonPhase(now),
		...sunTimes(ymd),
	};
}

/** Roman numerals for the volume line. */
export function roman(n: number): string {
	const map: [number, string][] = [
		[1000, 'M'], [900, 'CM'], [500, 'D'], [400, 'CD'], [100, 'C'], [90, 'XC'],
		[50, 'L'], [40, 'XL'], [10, 'X'], [9, 'IX'], [5, 'V'], [4, 'IV'], [1, 'I'],
	];
	let out = '';
	for (const [v, s] of map) while (n >= v) { out += s; n -= v; }
	return out;
}
