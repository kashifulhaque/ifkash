import * as THREE from 'three';

// One full day/night cycle in seconds of play (pause freezes the clock).
const DAY_LENGTH = 240;

// timeOfDay keyframes: 0 = midnight, 0.25 = dawn, 0.5 = noon, 0.75 = dusk.
type Phase = {
  t: number;
  sky: number;
  hemi: number;
  hemiIntensity: number;
  sun: number;
  sunIntensity: number;
  stars: number;
};
// "Night" never goes fully dark — it bottoms out at an indigo dusk so the
// world stays readable around the clock.
const PHASES: Phase[] = [
  { t: 0.0, sky: 0x3a3f6b, hemi: 0x55608f, hemiIntensity: 0.45, sun: 0x9fa8e8, sunIntensity: 0.4, stars: 0.5 },
  { t: 0.21, sky: 0x3a3f6b, hemi: 0x55608f, hemiIntensity: 0.45, sun: 0x9fa8e8, sunIntensity: 0.4, stars: 0.5 },
  { t: 0.27, sky: 0xf4a261, hemi: 0xffd9a0, hemiIntensity: 0.55, sun: 0xffb36b, sunIntensity: 0.8, stars: 0 },
  { t: 0.35, sky: 0x87ceeb, hemi: 0xfff8e7, hemiIntensity: 0.9, sun: 0xfff2cc, sunIntensity: 1.4, stars: 0 },
  { t: 0.65, sky: 0x87ceeb, hemi: 0xfff8e7, hemiIntensity: 0.9, sun: 0xfff2cc, sunIntensity: 1.4, stars: 0 },
  { t: 0.73, sky: 0xf4a261, hemi: 0xffd9a0, hemiIntensity: 0.55, sun: 0xffb36b, sunIntensity: 0.8, stars: 0 },
  { t: 0.79, sky: 0x3a3f6b, hemi: 0x55608f, hemiIntensity: 0.45, sun: 0x9fa8e8, sunIntensity: 0.4, stars: 0.5 },
  { t: 1.0, sky: 0x3a3f6b, hemi: 0x55608f, hemiIntensity: 0.45, sun: 0x9fa8e8, sunIntensity: 0.4, stars: 0.5 }
];

// Sky, fog, day/night lighting, stars and clouds that drift along with the
// player so the infinite world always has scenery overhead.
export class Environment {
  private clouds: THREE.Group[] = [];
  private sun: THREE.DirectionalLight;
  private hemi: THREE.HemisphereLight;
  private scene: THREE.Scene;
  private stars: THREE.Points;
  private starMat: THREE.PointsMaterial;
  private timeOfDay = 0.4; // start mid-morning
  private colA = new THREE.Color();
  private colB = new THREE.Color();

  constructor(scene: THREE.Scene) {
    this.scene = scene;
    scene.background = new THREE.Color(0x87ceeb);
    scene.fog = new THREE.Fog(0x87ceeb, 40, 95);

    this.hemi = new THREE.HemisphereLight(0xfff8e7, 0x6a8f3c, 0.9);
    scene.add(this.hemi);
    this.sun = new THREE.DirectionalLight(0xfff2cc, 1.4);
    this.sun.position.set(20, 35, 12);
    scene.add(this.sun);

    // Star dome: fixed points on a hemisphere, recentered on the player
    const starPos: number[] = [];
    for (let i = 0; i < 300; i++) {
      const a = Math.random() * Math.PI * 2;
      const e = Math.acos(Math.random()); // bias toward the zenith
      const r = 180;
      starPos.push(
        Math.cos(a) * Math.cos(e) * r,
        20 + Math.sin(e) * r * 0.5,
        Math.sin(a) * Math.cos(e) * r
      );
    }
    const starGeo = new THREE.BufferGeometry();
    starGeo.setAttribute('position', new THREE.Float32BufferAttribute(starPos, 3));
    this.starMat = new THREE.PointsMaterial({
      color: 0xffffff,
      size: 2,
      transparent: true,
      opacity: 0,
      fog: false,
      sizeAttenuation: false
    });
    this.stars = new THREE.Points(starGeo, this.starMat);
    this.stars.frustumCulled = false;
    this.stars.visible = false;
    scene.add(this.stars);

    const cloudMat = new THREE.MeshLambertMaterial({ color: 0xffffff, flatShading: true });
    for (let i = 0; i < 8; i++) {
      const cloud = new THREE.Group();
      for (let j = 0; j < 3; j++) {
        const puff = new THREE.Mesh(
          new THREE.IcosahedronGeometry(2 + Math.random() * 1.5, 0),
          cloudMat
        );
        puff.position.set(j * 2.4 - 2.4, Math.random(), Math.random() * 2);
        puff.scale.y = 0.5;
        cloud.add(puff);
      }
      cloud.position.set((Math.random() - 0.5) * 140, 24 + Math.random() * 10, (Math.random() - 0.5) * 140);
      cloud.userData.drift = 0.5 + Math.random();
      scene.add(cloud);
      this.clouds.push(cloud);
    }
  }

  // Lerp lighting between the two keyframes bracketing timeOfDay.
  private applyTimeOfDay() {
    const t = this.timeOfDay;
    let a = PHASES[0];
    let b = PHASES[PHASES.length - 1];
    for (let i = 0; i < PHASES.length - 1; i++) {
      if (t >= PHASES[i].t && t <= PHASES[i + 1].t) {
        a = PHASES[i];
        b = PHASES[i + 1];
        break;
      }
    }
    const span = b.t - a.t;
    const k = span > 0 ? (t - a.t) / span : 0;

    const sky = this.colA.setHex(a.sky).lerp(this.colB.setHex(b.sky), k);
    (this.scene.background as THREE.Color).copy(sky);
    (this.scene.fog as THREE.Fog).color.copy(sky);
    this.hemi.color.setHex(a.hemi).lerp(this.colB.setHex(b.hemi), k);
    this.hemi.intensity = a.hemiIntensity + (b.hemiIntensity - a.hemiIntensity) * k;
    this.sun.color.setHex(a.sun).lerp(this.colB.setHex(b.sun), k);
    this.sun.intensity = a.sunIntensity + (b.sunIntensity - a.sunIntensity) * k;
    const starOpacity = a.stars + (b.stars - a.stars) * k;
    this.starMat.opacity = starOpacity;
    this.stars.visible = starOpacity > 0.01;
  }

  update(dt: number, px: number, pz: number) {
    this.timeOfDay = (this.timeOfDay + dt / DAY_LENGTH) % 1;
    this.applyTimeOfDay();

    // Sun arcs overhead through the day; at night it sits low as "moonlight"
    const a = (this.timeOfDay - 0.25) * Math.PI * 2;
    this.sun.position.set(
      px + Math.cos(a) * 40,
      Math.max(8, Math.sin(a) * 45),
      pz + 12
    );
    this.sun.target.position.set(px, 0, pz);
    this.sun.target.updateMatrixWorld();

    this.stars.position.set(px, 0, pz);

    for (const cloud of this.clouds) {
      cloud.position.x += cloud.userData.drift * dt;
      // Wrap clouds around the player so they never run out
      if (cloud.position.x - px > 90) cloud.position.x = px - 90;
      if (cloud.position.x - px < -90) cloud.position.x = px + 90;
      if (cloud.position.z - pz > 90) cloud.position.z = pz - 90;
      if (cloud.position.z - pz < -90) cloud.position.z = pz + 90;
    }
  }
}
