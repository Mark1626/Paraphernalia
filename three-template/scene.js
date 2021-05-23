// Scene variables
var scene,
  camera,
  fieldOfView,
  aspectRatio,
  nearPlane,
  farPlane,
  HEIGHT,
  WIDTH,
  renderer,
  controls,
  container;

// Lights
var hemisphereLight, shadowLight;

const handleWindowResize = () => {
  HEIGHT = window.innerHeight;
  WIDTH = window.innerWidth;

  renderer.setSize(HEIGHT, WIDTH);
  camera.aspect = WIDTH / HEIGHT;
  camera.updateProjectionMatrix();
};

const createScene = () => {
  HEIGHT = window.innerHeight;
  WIDTH = window.innerWidth;

  scene = new THREE.Scene();
  if (ENABLE_FOG) scene.fog = new THREE.Fog(0xf7d9aa, 100, 950);

  aspectRatio = WIDTH / HEIGHT;
  fieldOfView = 60;
  nearPlane = 1;
  farPlane = 10000;
  camera = new THREE.PerspectiveCamera(fieldOfView, aspectRatio, nearPlane, farPlane);

  camera.position.x = 0;
  camera.position.y = 150;
  camera.position.z = 100;

  camera.applyMatrix4(new THREE.Matrix4().makeRotationX((45 * Math.PI)/180))

  renderer = new THREE.WebGLRenderer({
    alpha: true,
    antialias: ANTIALIAS,
  });
  controls = new THREE.OrbitControls(camera, renderer.domElement);

  renderer.setSize(WIDTH, HEIGHT);
  renderer.shadowMap.enabled = SHADOW_MAP;

  container = document.getElementById("celephais");
  container.appendChild(renderer.domElement);

  window.addEventListener("resize", handleWindowResize, false);
};

const createLights = () => {
  hemisphereLight = new THREE.HemisphereLight(0xaaaaaa, 0x000000, 0.9);
  shadowLight = new THREE.DirectionalLight(0xffffff, 0.9);

  shadowLight.position.set(0, 350, 350);
  shadowLight.castShadow = true;

  shadowLight.shadow.camera.left = -650;
  shadowLight.shadow.camera.right = 650;
  shadowLight.shadow.camera.top = 650;
  shadowLight.shadow.camera.bottom = -650;
  shadowLight.shadow.camera.near = 1;
  shadowLight.shadow.camera.far = 1000;

  if (SHADOW_MAP) {
    shadowLight.shadow.mapSize.width = 2048;
    shadowLight.shadow.mapSize.height = 2048;
  }

  scene.add(hemisphereLight);
  scene.add(shadowLight);
};
