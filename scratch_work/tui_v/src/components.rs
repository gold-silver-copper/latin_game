use crate::*;

#[derive(Default)]
pub struct ComponentHolder {
    pub positions: HashMap<EntityID, MyPoint>,
    pub ent_types: HashMap<EntityID, EntityType>,
    pub equipments: HashMap<EntityID, Equipment>,
    pub healths: HashMap<EntityID, Health>,
    pub genders: HashMap<EntityID, Gender>,
}

pub type EntSet = HashSet<EntityID>;
#[derive(Default)]
pub struct Equipment {
    pub equipped: EntSet,
    pub inventory: EntSet,
}

pub struct Health {
    pub current_health: i64,
    pub max_health: i64,
}

impl Default for Health {
    fn default() -> Self {
        Health {
            current_health: 3,
            max_health: 100,
        }
    }
}

impl App {
    pub fn get_unique_eid(&mut self) -> EntityID {
        self.entity_counter += 1;
        self.entity_counter.clone()
    }

    pub fn get_ent_type(&self, eid: &EntityID) -> EntityType {
        self.components.ent_types.get(eid).unwrap().clone()
    }

    pub fn get_entity_name(&self, subj: &EntityID) -> String {
        let ent_typ = self.get_ent_type(subj);

        let stringik = match ent_typ {
            EntityType::Human => "John".to_string(),
            EntityType::Item(itemik) => itemik.item_name(),
        };

        stringik
    }
    pub fn create_item(&mut self, item: ItemType) -> EntityID {
        let eid = self.get_unique_eid();

        self.components
            .ent_types
            .insert(eid.clone(), EntityType::Item(item));

        eid
    }

    pub fn set_ent_position(&mut self, eid: &EntityID, point: &MyPoint) {
        self.components.positions.insert(eid.clone(), point.clone());

        let voxik = self
            .game_map
            .get_mut_voxel_at(point)
            .expect("cant spawn ent in empty voxel");

        voxik.entity_set.insert(eid.clone());
    }

    pub fn spawn_item_at(&mut self, point: &MyPoint, item: ItemType) -> EntityID {
        let eid = self.create_item(item);
        self.set_ent_position(&eid, point);

        eid
    }
    pub fn spawn_human_at(&mut self, point: &MyPoint) -> EntityID {
        let eid = self.get_unique_eid();
        self.components.positions.insert(eid.clone(), point.clone());
        self.components
            .ent_types
            .insert(eid.clone(), EntityType::Human);
        self.components
            .genders
            .insert(eid.clone(), Gender::Masculine);
        self.components
            .equipments
            .insert(eid.clone(), Equipment::default());
        self.components
            .healths
            .insert(eid.clone(), Health::default());

        let voxik = self
            .game_map
            .get_mut_voxel_at(point)
            .expect("cant spawn ent in empty voxel");

        voxik.entity_set.insert(eid.clone());

        eid.clone()
    }
    pub fn spawn_player_at(&mut self, point: &MyPoint) -> EntityID {
        let pid = self.spawn_human_at(point);
        let iid = self.create_item(ItemType::Weapon(Weapon::Sword));
        let iid2 = self.create_item(ItemType::Clothing(Clothing::Toga));
        let iid3 = self.create_item(ItemType::Weapon(Weapon::Mace));

        let player_equip = self
            .components
            .equipments
            .get_mut(&pid)
            .expect("MUST HAVE QUEIP");
        player_equip.equipped.insert(iid);
        player_equip.inventory.insert(iid2);
        player_equip.inventory.insert(iid3);

        pid
    }
}
