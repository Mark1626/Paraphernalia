class Tree {
  constructor() {
    this.mesh = new THREE.Object3D();

    const leafMaterial = new THREE.MeshLambertMaterial({
      color: Colors.green,
    });

    const baseGeometry = new THREE.BoxGeometry(10, 20, 10);
    const baseMaterial = new THREE.MeshLambertMaterial({
      color: Colors.brown,
    });
    const base = new THREE.Mesh(baseGeometry, baseMaterial);
    base.castShadow = true;
    base.receiveShadow = true;
    this.mesh.add(base)

    const leaf1Geometry = new THREE.CylinderGeometry(1, 12*3, 12*3, 4 );
    const leaf1 = new THREE.Mesh(leaf1Geometry, leafMaterial);
    leaf1.castShadow = true;
    leaf1.receiveShadow = true;
    leaf1.position.y = 20;
    this.mesh.add(leaf1);

    const leaf2Geometry = new THREE.CylinderGeometry( 1, 9*3, 9*3, 4 );
    const leaf2 = new THREE.Mesh(leaf2Geometry, leafMaterial);
    leaf2.castShadow = true;
    leaf2.receiveShadow = true;
    leaf2.position.y = 40;
    this.mesh.add(leaf2);

    const leaf3Geometry = new THREE.CylinderGeometry( 1, 6*3, 6*3, 4);
    const leaf3 = new THREE.Mesh(leaf3Geometry, leafMaterial);
    leaf3.castShadow = true;
    leaf3.receiveShadow = true;
    leaf3.position.y = 55;
    this.mesh.add(leaf3);
  }
}

class Forest {
  constructor(trees) {
    this.mesh = new THREE.Object3D();
    this.trees = trees;

    for (let i = 0; i < this.trees; i += 1) {
      const t = new Tree();
      t.mesh.position.y = 505;
      t.mesh.position.x = -300 + (Math.random() * 500);
      t.mesh.position.z = -(Math.random() * 600);

      const s = 0.3 + 0.75*Math.random();
      t.mesh.scale.set(s, s, s);

      this.mesh.add(t.mesh);
    }
  }
}
