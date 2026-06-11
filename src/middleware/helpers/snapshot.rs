use rbx_dom_weak::{
	types::{Ref, Variant},
	AHashMap, Instance, WeakDom,
};

use crate::core::{meta::Meta, snapshot::Snapshot};

// Based on Rojo's InstanceSnapshot::from_tree (https://github.com/rojo-rbx/rojo/blob/master/src/snapshot/instance_snapshot.rs#L105)
pub fn snapshot_from_dom(dom: WeakDom, id: Ref) -> Snapshot {
	let (_, mut raw_dom) = dom.into_raw();

	let mut instances: AHashMap<Ref, Instance> = AHashMap::new();
	let mut ref_map: AHashMap<Ref, Ref> = AHashMap::new();

	fn collect(
		id: Ref,
		raw_dom: &mut AHashMap<Ref, Instance>,
		instances: &mut AHashMap<Ref, Instance>,
		ref_map: &mut AHashMap<Ref, Ref>,
	) {
		let instance = raw_dom
			.remove(&id)
			.expect("Provided ID does not exist in the current DOM");

		ref_map.insert(id, Ref::new());

		for &child_id in instance.children() {
			collect(child_id, raw_dom, instances, ref_map);
		}

		instances.insert(id, instance);
	}

	collect(id, &mut raw_dom, &mut instances, &mut ref_map);

	fn build(id: Ref, instances: &mut AHashMap<Ref, Instance>, ref_map: &AHashMap<Ref, Ref>) -> Snapshot {
		let mut instance = instances
			.remove(&id)
			.expect("Provided ID does not exist in the current DOM");

		for value in instance.properties.values_mut() {
			if let Variant::Ref(reference) = value {
				*value = Variant::Ref(ref_map.get(reference).copied().unwrap_or_else(Ref::none));
			}
		}

		let children = instance
			.children()
			.iter()
			.map(|&child_id| build(child_id, instances, ref_map))
			.collect();

		let mut meta = Meta::new();

		if instance.class == "MeshPart" {
			meta.set_mesh_source(super::save_mesh(&instance.properties));
		}

		let mut snapshot = Snapshot::new()
			.with_meta(meta)
			.with_name(&instance.name)
			.with_class(&instance.class)
			.with_properties(instance.properties)
			.with_children(children);

		snapshot.ref_id = ref_map[&id];
		snapshot
	}

	build(id, &mut instances, &ref_map)
}

#[cfg(test)]
mod tests {
	use rbx_dom_weak::{InstanceBuilder, Ustr};

	use super::*;

	#[test]
	fn remaps_internal_refs() {
		let hitbox = InstanceBuilder::new("Part").with_name("Hitbox");
		let hitbox_ref = hitbox.referent();

		let weld = InstanceBuilder::new("Weld")
			.with_name("Weld")
			.with_property("Part0", Variant::Ref(hitbox_ref));

		let model = InstanceBuilder::new("Model")
			.with_name("Model")
			.with_property("PrimaryPart", Variant::Ref(hitbox_ref))
			.with_child(hitbox)
			.with_child(weld);

		let model_ref = model.referent();

		let dom = WeakDom::new(model);

		let snapshot = snapshot_from_dom(dom, model_ref);

		let hitbox_snapshot = snapshot.children.iter().find(|child| child.name == "Hitbox").unwrap();
		let weld_snapshot = snapshot.children.iter().find(|child| child.name == "Weld").unwrap();

		assert_eq!(
			snapshot.properties.get(&Ustr::from("PrimaryPart")),
			Some(&Variant::Ref(hitbox_snapshot.ref_id))
		);
		assert_eq!(
			weld_snapshot.properties.get(&Ustr::from("Part0")),
			Some(&Variant::Ref(hitbox_snapshot.ref_id))
		);

		assert_ne!(hitbox_snapshot.ref_id, hitbox_ref);
		assert_ne!(snapshot.ref_id, model_ref);
		assert_eq!(snapshot.id, Ref::none());
	}
}
