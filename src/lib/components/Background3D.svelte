<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import * as THREE from "three";

    let canvasContainer: HTMLDivElement;
    let renderer: THREE.WebGLRenderer;
    let animationFrameId: number;

    onMount(() => {
        // Scene setup
        const scene = new THREE.Scene();

        // Camera setup
        const camera = new THREE.PerspectiveCamera(
            75,
            window.innerWidth / window.innerHeight,
            0.1,
            1000,
        );
        camera.position.z = 30;

        // Renderer setup
        renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
        renderer.setSize(window.innerWidth, window.innerHeight);
        renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        canvasContainer.appendChild(renderer.domElement);

        // Particles array
        const particlesGeometry = new THREE.BufferGeometry();
        const particlesCount = 900;

        const posArray = new Float32Array(particlesCount * 3);

        for (let i = 0; i < particlesCount * 3; i++) {
            posArray[i] = (Math.random() - 0.5) * 120;
            if (i % 3 === 1) posArray[i] = (Math.random() - 0.5) * 120; // y
            if (i % 3 === 2) posArray[i] = (Math.random() - 0.5) * 120; // z
        }

        particlesGeometry.setAttribute(
            "position",
            new THREE.BufferAttribute(posArray, 3),
        );

        const particlesMaterial = new THREE.PointsMaterial({
            size: 0.25,
            color: 0x888888,
            transparent: true,
            opacity: 0.8,
            blending: THREE.AdditiveBlending,
        });

        const particlesMesh = new THREE.Points(
            particlesGeometry,
            particlesMaterial,
        );
        scene.add(particlesMesh);

        // Fancy Shape 1: Outer Icosahedron
        const geoOuter = new THREE.IcosahedronGeometry(12, 1);
        const matOuter = new THREE.MeshBasicMaterial({
            color: 0x444444,
            wireframe: true,
            transparent: true,
            opacity: 0.15,
        });
        const meshOuter = new THREE.Mesh(geoOuter, matOuter);
        scene.add(meshOuter);

        // Fancy Shape 2: Inner Torus Knot
        const geoInner = new THREE.TorusKnotGeometry(4, 1.2, 100, 16);
        const matInner = new THREE.MeshBasicMaterial({
            color: 0x666666,
            wireframe: true,
            transparent: true,
            opacity: 0.2,
        });
        const meshInner = new THREE.Mesh(geoInner, matInner);
        scene.add(meshInner);

        // Mouse interaction variables
        let mouseX = 0;
        let mouseY = 0;
        let targetZ = 30;

        const onDocumentMouseMove = (event: MouseEvent) => {
            mouseX = (event.clientX - window.innerWidth / 2) * 0.05;
            mouseY = (event.clientY - window.innerHeight / 2) * 0.05;
            targetZ = 30 + Math.abs(mouseX) * 0.1; // Gentle zoom based on mouse distance
        };

        document.addEventListener("mousemove", onDocumentMouseMove);

        // Resize handler
        const onWindowResize = () => {
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
        };

        window.addEventListener("resize", onWindowResize);

        // Animation loop
        const clock = new THREE.Clock();

        const animate = () => {
            animationFrameId = requestAnimationFrame(animate);

            const elapsedTime = clock.getElapsedTime();

            // Dynamic color shifting using HSL
            const hue1 = (Math.sin(elapsedTime * 0.2) + 1) / 2;
            const hue2 = (Math.cos(elapsedTime * 0.15) + 1) / 2;
            matOuter.color.setHSL(hue1, 0.4, 0.5);
            matInner.color.setHSL(hue2, 0.6, 0.6);
            particlesMaterial.color.setHSL((hue1 + hue2) / 2, 0.5, 0.7);

            // Rotate particles
            particlesMesh.rotation.y = elapsedTime * 0.05;
            particlesMesh.rotation.x = elapsedTime * 0.02;

            // Rotate and gently pulsate outer shape
            meshOuter.rotation.y += 0.002;
            meshOuter.rotation.x += 0.001;
            const scaleOuter = 1 + Math.sin(elapsedTime) * 0.05;
            meshOuter.scale.set(scaleOuter, scaleOuter, scaleOuter);

            // Rotate and aggressively pulsate inner shape
            meshInner.rotation.y -= 0.005;
            meshInner.rotation.z += 0.003;
            const scaleInner = 1 + Math.cos(elapsedTime * 2) * 0.1;
            meshInner.scale.set(scaleInner, scaleInner, scaleInner);

            // Interactive camera based on mouse
            camera.position.x += (mouseX - camera.position.x) * 0.05;
            camera.position.y += (-mouseY - camera.position.y) * 0.05;
            camera.position.z += (targetZ - camera.position.z) * 0.02;
            camera.lookAt(scene.position);

            renderer.render(scene, camera);
        };

        animate();

        // Cleanup function assigned to onDestroy
        onDestroy(() => {
            window.removeEventListener("resize", onWindowResize);
            document.removeEventListener("mousemove", onDocumentMouseMove);
            if (animationFrameId) cancelAnimationFrame(animationFrameId);

            // Remove canvas
            if (
                renderer &&
                renderer.domElement &&
                renderer.domElement.parentNode
            ) {
                renderer.domElement.parentNode.removeChild(renderer.domElement);
            }

            // Dispose geometries and materials
            geoOuter.dispose();
            matOuter.dispose();
            geoInner.dispose();
            matInner.dispose();
            particlesGeometry.dispose();
            particlesMaterial.dispose();
            renderer.dispose();
        });
    });
</script>

<div class="webgl-container" bind:this={canvasContainer}></div>

<style>
    .webgl-container {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        z-index: -1;
        pointer-events: none; /* Let clicks pass through to the content below */
        overflow: hidden;
    }
</style>
