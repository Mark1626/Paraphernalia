class Land {
  constructor() {
    const geometry = new THREE.PlaneGeometry(600, 600, 1);
    geometry.applyMatrix4(new THREE.Matrix4().makeRotationX(-Math.PI/2));
    const material = new THREE.MeshPhongMaterial({
      color: Colors.lightgreen,
      flatShading: true,
    });
    this.mesh = new THREE.Mesh(geometry, material);
    this.mesh.receiveShadow = true;
  }
}

class Sun {
  constructor() {
    this.mesh = new THREE.Object3D();
    const geometry = new THREE.SphereGeometry(400, 20, 10);
    const material = new THREE.MeshPhongMaterial({
      color: Colors.yellow,
      flatShading: true,
    });
    const sun = new THREE.Mesh(geometry, material);
    sun.cashShadow = false;
    sun.receiveShadow = false;
    this.mesh.add(sun);
  }
}
