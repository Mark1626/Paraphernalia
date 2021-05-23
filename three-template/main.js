let land, sun, world, forest;
let offSet = -600;

const createPrimitives = () => {
  land = new Land();
  land.mesh.position.x = 0;
  land.mesh.position.y = -100;
  land.mesh.position.z = -100;
  scene.add(land.mesh);

  sun = new Sun();
  sun.mesh.scale.set(1, 1, 0.3);
  sun.mesh.position.set(0, -30, -850);
  scene.add(sun.mesh);
};

const createEcosystem = () => {
  forest = new Forest(100);
  forest.mesh.position.y = offSet;
  scene.add(forest.mesh)
}

const update = () => {
  renderer.render(scene, camera);
  controls.update();
  requestAnimationFrame(update);
};

const init = () => {
  createScene();
  createLights();
  createPrimitives();
  createEcosystem();
  update();
};

window.addEventListener("load", init, false);
