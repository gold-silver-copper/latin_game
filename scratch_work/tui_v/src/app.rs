use std::collections::VecDeque;

use rand::Rng;

use crate::*;
// ANCHOR: app

pub struct App {
    pub inflector: ISV,
    pub entity_counter: i64,
    pub components: ComponentHolder,
    pub input_state: InputState,
    pub action_results: Vec<ActionResult>,
    pub action_result_strings: VecDeque<String>,
    pub selected_menu: ItemVecType,
    pub item_list_state: ListState,
    pub exit: bool,
    pub game_map: GameMap,
    pub action_vec: ActionVec,
    pub local_player_id: EntityID,
    pub seen_tiles: HashSet<BracketPoint>,
    pub small_rng: SmallRng,
}
impl Default for App {
    fn default() -> Self {
        let mut inflector = ISV::default();
        inflector.initialize_dictionary("isv_words.csv");
        App {
            inflector,
            entity_counter: 0,
            components: ComponentHolder::default(),
            input_state: InputState::default(),
            action_results: Vec::new(),
            action_result_strings: VecDeque::new(),
            selected_menu: ItemVecType::default(),
            item_list_state: ListState::default(),
            exit: false,
            game_map: GameMap::default(),
            action_vec: ActionVec::default(),
            local_player_id: 0,
            seen_tiles: HashSet::new(),
            small_rng: SmallRng::seed_from_u64(1),
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        self.init();
        while !self.exit {
            self.update_seen_tiles();
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
            if !self.action_vec.is_empty() {
                self.handle_ai();
                self.handle_actions()?;
            }
            self.handle_deaths();
            self.gen_action_result_strings();
            self.reload_ui();
        }
        Ok(())
    }

    pub fn handle_deaths(&mut self) {
        let mut healths_to_remove = Vec::new();
        for (eid, health) in &self.components.healths {
            if health.current_health <= 0 {
                self.action_results.push(ActionResult::Success(
                    GameAction::Death(eid.clone()),
                    SuccessType::Normal,
                ));
                healths_to_remove.push(eid.clone());
            }
        }
        for eid in &healths_to_remove {
            if let Some(e_pos) = self.components.positions.remove(eid) {
                if let Some(voxik) = self.game_map.get_mut_voxel_at(&e_pos) {
                    remove_ent_from_vec(&mut voxik.entity_set, eid);

                    if let Some(equi) = self.components.equipments.remove(eid) {
                        for ano in equi.equipped {
                            voxik.entity_set.push(ano);
                        }
                        for ano in equi.inventory {
                            voxik.entity_set.push(ano);
                        }
                        if equi.arrows > 0 {
                            self.spawn_item_at(&e_pos, ItemType::Ammo(Ammo::Strěla(equi.arrows)));
                        }
                        if equi.darts > 0 {
                            self.spawn_item_at(&e_pos, ItemType::Ammo(Ammo::Drotik(equi.darts)));
                        }
                        if equi.javelins > 0 {
                            self.spawn_item_at(&e_pos, ItemType::Ammo(Ammo::Oščěp(equi.javelins)));
                        }
                        if equi.bullets > 0 {
                            self.spawn_item_at(&e_pos, ItemType::Ammo(Ammo::Kulja(equi.bullets)));
                        }
                    }
                }
            }
            self.components.healths.remove(&eid);
        }
    }

    pub fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn init(&mut self) {
        let pik = (100, 100);

        self.local_player_id = self.spawn_player_at(&pik);

        for x in 20..MAP_SIZE {
            for y in 20..MAP_SIZE {
                if self.small_rng.gen_bool(0.02) {
                    for meow in Profession::iter() {
                        if self.small_rng.gen_bool(0.02) {
                            self.spawn_human_at(&(x, y), meow);
                            break;
                        }
                    }
                }
            }
        }
        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                if self.small_rng.gen_bool(0.02) {
                    for meow in AnimalType::iter() {
                        if self.small_rng.gen_bool(0.02) {
                            self.spawn_animal_at(&meow, &(x, y));
                            break;
                        }
                    }
                }
            }
        }
        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                if self.small_rng.gen_bool(0.02) {
                    for meow in Consumable::iter() {
                        if self.small_rng.gen_bool(0.02) {
                            self.spawn_item_at(&(x, y), ItemType::Consumable(meow));
                            break;
                        }
                    }
                }
            }
        }
        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                if self.small_rng.gen_bool(0.02) {
                    for meow in Ammo::iter() {
                        if self.small_rng.gen_bool(0.02) {
                            let amountik = self.small_rng.gen_range(1..50);
                            match meow {
                                Ammo::Kulja(_) => self
                                    .spawn_item_at(&(x, y), ItemType::Ammo(Ammo::Kulja(amountik))),
                                Ammo::Strěla(_) => self
                                    .spawn_item_at(&(x, y), ItemType::Ammo(Ammo::Strěla(amountik))),
                                Ammo::Oščěp(_) => self
                                    .spawn_item_at(&(x, y), ItemType::Ammo(Ammo::Oščěp(amountik))),
                                Ammo::Drotik(_) => self
                                    .spawn_item_at(&(x, y), ItemType::Ammo(Ammo::Drotik(amountik))),
                            };

                            break;
                        }
                    }
                }
            }
        }

        self.spawn_item_at(&(5, 6), ItemType::Ammo(Ammo::Drotik(50)));

        self.reload_ui();
    }

    pub fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    pub fn manage_item_vec_input(&self) -> (bool, EntityID) {
        let ground_vec = self.ground_item_vec_at_ent(&self.local_player_id);
        let equi_vec = self.equipped_item_vec_of_ent(&self.local_player_id);
        let inv_vec = self.inventory_item_vec_of_ent(&self.local_player_id);

        if let Some(sid) = self.item_list_state.selected() {
            if self.input_state == InputState::Inventory {
                let moop = match self.selected_menu {
                    ItemVecType::Equipped => equi_vec.get(sid),
                    ItemVecType::Inventory => inv_vec.get(sid),
                    ItemVecType::Ground => ground_vec.get(sid),
                };

                if let Some(id_to_select) = moop {
                    let id_to_select = id_to_select.clone();

                    return (true, id_to_select);
                }
            } else if self.input_state == InputState::RangedAttack {
                let vecik = self.ranged_attackable_ents(&self.local_player_id);
                if let Some(moop) = vecik.get(sid) {
                    return (true, moop.clone());
                }
            }
        }

        (false, 0)
    }

    pub fn handle_ai(&mut self) {
        let mut conscious_ents = Vec::new();
        let mut stupid_ents = Vec::new();
        let ents_visible_from_player = self.generate_visible_ents_from_ent(&self.local_player_id);
        let lp_pos = self
            .components
            .positions
            .get(&self.local_player_id)
            .unwrap_or(&(0, 0))
            .clone();

        for boop in &self.components.ent_types {
            if (boop.0 != &self.local_player_id) {
                match boop.1 {
                    EntityType::Human(_) => conscious_ents.push(boop.0.clone()),
                    EntityType::Animal(_) => stupid_ents.push(boop.0.clone()),
                    _ => (),
                }
            }
        }

        for meow in conscious_ents {
            if ents_visible_from_player.contains(&meow) {
                if let Some(ent_pos) = self.components.positions.get(&meow) {
                    let x_dif = ent_pos.0 - lp_pos.0;
                    let y_dif = ent_pos.1 - lp_pos.1;

                    if (x_dif >= 0) {
                        if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::West));
                        } else if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::SouthWest));
                        } else {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::NorthWest));
                        }
                    } else if (x_dif < 0) {
                        if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::East));
                        } else if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::SouthEast));
                        } else {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::NorthEast));
                        }
                    }
                    if (y_dif >= 0) {
                        if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::South));
                        } else if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::SouthWest));
                        } else {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::SouthEast));
                        }
                    } else if (y_dif < 0) {
                        if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::North));
                        } else if self.small_rng.gen_bool(0.8) {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::NorthEast));
                        } else {
                            self.action_vec
                                .push(GameAction::Go(meow, CardinalDirection::NorthWest));
                        }
                    }
                }
            } /*else {
                  if self.small_rng.gen_bool(0.3) {
                      self.action_vec
                          .push(GameAction::Go(meow, CardinalDirection::West));
                  } else if self.small_rng.gen_bool(0.3) {
                      self.action_vec
                          .push(GameAction::Go(meow, CardinalDirection::East));
                  } else if self.small_rng.gen_bool(0.3) {
                      self.action_vec
                          .push(GameAction::Go(meow, CardinalDirection::South));
                  } else if self.small_rng.gen_bool(0.3) {
                      self.action_vec
                          .push(GameAction::Go(meow, CardinalDirection::North));
                  }
              } */
        }
        for meow in stupid_ents {
            if ents_visible_from_player.contains(&meow) {
                if self.small_rng.gen_bool(0.3) {
                    self.action_vec
                        .push(GameAction::Go(meow, CardinalDirection::West));
                } else if self.small_rng.gen_bool(0.3) {
                    self.action_vec
                        .push(GameAction::Go(meow, CardinalDirection::East));
                } else if self.small_rng.gen_bool(0.3) {
                    self.action_vec
                        .push(GameAction::Go(meow, CardinalDirection::South));
                } else if self.small_rng.gen_bool(0.3) {
                    self.action_vec
                        .push(GameAction::Go(meow, CardinalDirection::North));
                }
            }
        }
    }

    pub fn reload_ui(&mut self) {
        self.game_map.ent_types_copy = self.components.ent_types.clone();
        match self.input_state {
            InputState::Inventory => {
                let boopik = match self.selected_menu {
                    ItemVecType::Equipped => {
                        self.equipped_item_vec_of_ent(&self.local_player_id).len()
                    }
                    ItemVecType::Inventory => {
                        self.inventory_item_vec_of_ent(&self.local_player_id).len()
                    }
                    ItemVecType::Ground => self.ground_item_vec_at_ent(&self.local_player_id).len(),
                };

                if let Some(sel_len) = self.item_list_state.selected_mut() {
                    if ((*sel_len >= boopik) && (boopik > 0)) {
                        *sel_len = boopik - 1;
                    }
                }
            }
            InputState::RangedAttack => {
                let boopik = self.ranged_attackable_ents(&self.local_player_id).len();
                if let Some(sel_len) = self.item_list_state.selected_mut() {
                    if ((*sel_len >= boopik) && (boopik > 0)) {
                        *sel_len = boopik - 1;
                    }
                }
            }

            _ => (),
        }
    }

    pub fn handle_actions(&mut self) -> Result<()> {
        let a_map = self.action_vec.clone();
        self.action_vec = Vec::new();

        for act in a_map {
            let act_result = match act {
                GameAction::Go(subj_id, cd) => {
                    (subj_id.clone(), self.handle_movement(&subj_id, &cd))
                }
                GameAction::Drop(subj_id, obj_id) => {
                    (subj_id.clone(), self.drop_item_from_inv(&subj_id, &obj_id))
                }
                GameAction::PickUp(subj_id, obj_id) => (
                    subj_id.clone(),
                    self.pickup_item_from_ground(&subj_id, &obj_id),
                ),
                GameAction::Equip(subj_id, obj_id) => {
                    (subj_id.clone(), self.equip_item_from_inv(&subj_id, &obj_id))
                }
                GameAction::UnEquip(subj_id, obj_id) => (
                    subj_id.clone(),
                    self.unequip_item_from_equipped(&subj_id, &obj_id),
                ),
                GameAction::RangedAttack(subj_id, obj_id) => {
                    (subj_id.clone(), self.ranged_attack(&subj_id, &obj_id))
                }
                GameAction::Wait(subj_id) => (subj_id.clone(), self.handle_wait(&subj_id)),
                GameAction::BumpAttack(subj_id, obj_id) => {
                    (subj_id.clone(), self.bump_attack(&subj_id, &obj_id))
                }
                GameAction::Death(subj_id) => (
                    subj_id.clone(),
                    ActionResult::Success(GameAction::Death(subj_id), SuccessType::Normal),
                ),
                GameAction::Consume(subj_id, consum) => (
                    subj_id.clone(),
                    ActionResult::Success(
                        GameAction::Consume(subj_id, consum),
                        SuccessType::Normal,
                    ),
                ),
            };

            if (act_result.0 == self.local_player_id)
                || (self
                    .generate_visible_ents_from_ent(&self.local_player_id)
                    .contains(&act_result.0))
            {
                self.action_results.push(act_result.1);
            }
        }

        Ok(())
    }
    pub fn title_block(title: &str) -> Block {
        let title = Title::from(title).alignment(Alignment::Center);
        Block::new()
            .borders(Borders::NONE)
            .padding(Padding::vertical(0))
            .title(title)
            .fg(Color::Blue)
    }

    pub fn ranged_attackable_ents(&self, subj: &EntityID) -> Vec<EntityID> {
        let mut new_vec = Vec::new();

        if let Some(subj_pos) = self.components.positions.get(subj) {
            let visible_ents = self.generate_visible_ents_from_ent(subj);
            for entik in visible_ents {
                let typik = self.get_ent_type(&entik);
                if typik.is_attackable() {
                    if let Some(obj_pos) = self.components.positions.get(subj) {
                        if self.game_map.line_from_point_to_point_is_unblocked(
                            subj_pos,
                            obj_pos,
                            &self.components.ent_types,
                        ) {
                            new_vec.push(entik);
                        }
                    }
                }
            }
        }
        new_vec
    }

    pub fn render_const_info(&self, area: Rect, buf: &mut Buffer) {
        let const_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(4),
            ],
        )
        .split(area);
        let mut cur_hel = 0;
        let max_hel = self.max_health_of_ent(&self.local_player_id);

        if let Some(stats) = self.components.stats.get(&self.local_player_id) {
            let stats_string = format!(
                "Sila-{} Bystrst-{} Råzum-{}",
                stats.strength, stats.speed, stats.intelligence
            );
            Paragraph::new(Text::from(stats_string))
                .on_black()
                .centered()
                .render(const_layout[1], buf);
        }

        let ent_name = self.get_entity_name(&self.local_player_id);
        if let Some(health) = self.components.healths.get(&self.local_player_id) {
            cur_hel = health.current_health.clone();
        }
        let title = App::title_block(&ent_name);

        let label = Span::styled(
            format!("{}/{}", cur_hel, max_hel),
            Style::new().bold().fg(Color::Black),
        );
        Gauge::default()
            .block(title)
            .gauge_style(Style::new().fg(Color::Green).bg(Color::LightRed))
            .ratio(cur_hel as f64 / max_hel as f64)
            .label(label)
            .render(const_layout[0], buf);

        if let Some(equi) = self.components.equipments.get(&self.local_player_id) {
            let line_one = format!("Strěly:{}  Kulje:{}", equi.arrows, equi.bullets);
            let line_two = format!("Oščěpy:{}  Drotiki:{}", equi.javelins, equi.darts);
            let standart = vec![Line::from(line_one), Line::from(line_two)];

            let lines = (Text::from(standart));

            Paragraph::new(Text::from(lines))
                .on_black()
                .block(Block::bordered().title("Amunicija"))
                .centered()
                .render(const_layout[4], buf);
        }

        let mut weapons = Vec::new();
        let mut armor = Vec::new();
        let mut weapon_string = String::from("Pęsti");
        let mut armor_string = String::from("Ničto");
        if let Some(player_equip) = self.components.equipments.get(&self.local_player_id) {
            if player_equip.equipped.is_empty() {
            } else {
                for (item) in player_equip.equipped.iter() {
                    let item_type = self
                        .components
                        .ent_types
                        .get(item)
                        .expect("EVERY ITEM MUST HAVE AN ENTITY TYPE");

                    match item_type {
                        EntityType::Human(_) => (),
                        EntityType::Animal(_) => (),
                        EntityType::Item(itemik) => match itemik {
                            ItemType::Weapon(_) => weapons.push(itemik.item_name()),

                            ItemType::Clothing(_) => armor.push(itemik.item_name()),
                            _ => (),
                        },
                    }
                }
            }
            if weapons.len() == 1 {
                weapon_string = weapons[0].clone();
            }
            if weapons.len() == 2 {
                weapon_string = format!("{} i {}", weapons[0].clone(), weapons[1].clone())
            }

            if armor.len() > 0 {
                armor_string = String::from(armor[0].clone());
                for thing in 1..armor.len() {
                    armor_string.push_str(&format!(", {}", armor[thing].clone()));
                }
            }

            Paragraph::new(Text::from(weapon_string))
                .on_black()
                .block(Block::bordered().title("Orųžeńje"))
                .centered()
                .render(const_layout[2], buf);
            Paragraph::new(Text::from(armor_string))
                .on_black()
                .block(Block::bordered().title("Odědža"))
                .wrap(Wrap { trim: true })
                .render(const_layout[3], buf);
        }
    }

    pub fn render_key_hints(&self, area: Rect, buf: &mut Buffer) {
        let string = match self.input_state {
            InputState::Basic => {
                format! {"[{RANGED_ATTACK}]-oddaljena ataka  [{INVENTORY_MENU}]-rukzak  [{WAIT_KEY}]-čekati  [{CURSOR_UP}{CURSOR_LEFT}{CURSOR_DOWN}{CURSOR_RIGHT} + {CURSOR_UP_LEFT}{CURSOR_UP_RIGHT}{CURSOR_DOWN_LEFT}{CURSOR_DOWN_RIGHT}]-dvigati sę  [{QUIT_BACK}]-vyjdti iz igry"}
            }
            InputState::Inventory => {
                format! {"[{CURSOR_LEFT}/{CURSOR_RIGHT}]-měnjati menju  [{CURSOR_UP}/{CURSOR_DOWN}]-izbirati věć  [{DROP_UNEQUIP_ACTION}]-odkladati/opustiti  [{PICKUP_EQUIP_ACTION}]-podbirati/equipirovati/jesti [{INVENTORY_MENU}]-zakryti rukzak"}
            }
            InputState::RangedAttack => {
                format! {"[{CURSOR_RIGHT}]-atakovati  [{CURSOR_LEFT}]-měnjati orųžje  [{CURSOR_UP}/{CURSOR_DOWN}]-izbirati vråga  [{RANGED_ATTACK}]-izključiti režim oddaljenoj ataky",}
            }
        };
        Paragraph::new(Text::from(string))
            .on_gray()
            .black()
            .centered()
            .render(area, buf);
    }

    pub fn gen_symbol_name_line_vec(&self, id_vec: &Vec<EntityID>) -> Vec<Line> {
        let mut visible_lines = Vec::new();
        let visible_symbols = self.gen_item_symbol_vec(id_vec);
        let visible_names = self.gen_item_name_vec(id_vec);
        for boopik in 0..id_vec.len() {
            let stringik = format! {"{}  {}",visible_symbols[boopik],visible_names[boopik]};
            visible_lines.push(Line::from(stringik));
        }
        visible_lines
    }

    pub fn render_info_paragraph(&self, area: Rect, buf: &mut Buffer) {
        let visible_lines = self
            .gen_symbol_name_line_vec(&self.generate_visible_ents_from_ent(&self.local_player_id));

        let lines = (Text::from(visible_lines));

        Paragraph::new(Text::from(lines))
            .on_black()
            .block(Block::bordered().title("Ty vidiš........."))
            .render(area, buf);
    }

    pub fn gen_action_result_strings(&mut self) {
        let meow = self.action_results.clone();
        self.action_results.clear();
        for boop in meow {
            let linija = self.generate_action_result_string(boop.clone());
            if linija != String::from("") {
                self.action_result_strings.push_front(linija);
            }
        }
        self.action_result_strings.truncate(100);
    }

    pub fn render_event_paragraph(&self, area: Rect, buf: &mut Buffer) {
        let mut line_vec = Vec::new();
        let mut events_copy = self.action_result_strings.clone();

        for _ in 0..20 {
            let boop = events_copy.pop_front();
            if let Some(actik) = boop {
                line_vec.push(Line::from(actik));
            }
        }

        //  line_vec.reverse();
        let lines = (Text::from(line_vec));

        Paragraph::new(Text::from(lines))
            .on_black()
            .block(Block::bordered())
            .render(area, buf);
    }

    pub fn render_item_list(&self, title: &str, itemvectype: ItemVecType) -> List {
        let wut = match itemvectype {
            ItemVecType::Equipped => {
                self.gen_item_name_vec(&self.equipped_item_vec_of_ent(&self.local_player_id))
            }
            ItemVecType::Inventory => {
                self.gen_item_name_vec(&self.inventory_item_vec_of_ent(&self.local_player_id))
            }
            ItemVecType::Ground => {
                self.gen_item_name_vec(&self.ground_item_vec_at_ent(&self.local_player_id))
            }
        };

        let list = List::new(wut)
            .block(Block::bordered().title(title.to_string()))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">")
            .repeat_highlight_symbol(true);

        list
    }
    pub fn render_ranged_attackable_list(&self) -> List {
        let mut title = "".to_string();
        if let Some(equi) = self.components.equipments.get(&self.local_player_id) {
            title = format!("{}", equi.ranged_weapon);
        }

        let wut = self.ranged_attackable_ents(&self.local_player_id);
        let listik = self.gen_symbol_name_line_vec(&wut);

        let list = List::new(listik)
            .block(Block::bordered().title(title.to_string()))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">")
            .repeat_highlight_symbol(true);

        list
    }

    pub fn gen_item_name_vec(&self, id_vec: &Vec<EntityID>) -> Vec<String> {
        let mut itemnamevec = Vec::new();

        for itik in id_vec.iter() {
            let typik = self
                .components
                .ent_types
                .get(itik)
                .expect("ent type must have");
            let itname = match typik {
                EntityType::Human(_) => self.get_entity_name(itik),
                EntityType::Animal(anim) => format!("{anim}"),
                EntityType::Item(itemik) => itemik.item_name(),
            };
            itemnamevec.push(itname);
        }
        itemnamevec
    }
    pub fn gen_item_symbol_vec(&self, id_vec: &Vec<EntityID>) -> Vec<String> {
        let mut itemnamevec = Vec::new();

        for itik in id_vec.iter() {
            let typik = self
                .components
                .ent_types
                .get(itik)
                .expect("ent type must have");

            itemnamevec.push(typik.symbol());
        }
        itemnamevec
    }
    pub fn inventory_item_vec_of_ent(&self, eid: &EntityID) -> Vec<EntityID> {
        let mut evec = Vec::new();
        if let Some(ent_equi) = self.components.equipments.get(eid) {
            for itemik in ent_equi.inventory.iter() {
                evec.push(itemik.clone());
            }
        }

        evec
    }

    pub fn equipped_item_vec_of_ent(&self, eid: &EntityID) -> Vec<EntityID> {
        let mut evec = Vec::new();
        if let Some(ent_equi) = self.components.equipments.get(eid) {
            for itemik in ent_equi.equipped.iter() {
                evec.push(itemik.clone());
            }
        }

        evec
    }

    pub fn ground_item_vec_at_ent(&self, eid: &EntityID) -> Vec<EntityID> {
        let mut evec = Vec::new();

        if let Some(ent_pos) = self.components.positions.get(eid) {
            let ent_vox = self.game_map.get_voxel_at(ent_pos).unwrap();

            for boop in ent_vox.entity_set.iter() {
                let booptype = self.get_ent_type(boop);
                match booptype {
                    EntityType::Human(_) => {}
                    EntityType::Animal(_) => {}
                    EntityType::Item(x) => {
                        evec.push(boop.clone());
                    }
                }
            }
        }

        evec
    }
    pub fn update_seen_tiles(&mut self) {
        if let Some(my_pos) = self.components.positions.get(&self.local_player_id) {
            let fov = field_of_view_set(
                BracketPoint {
                    x: my_pos.0,
                    y: my_pos.1,
                },
                FOV_RANGE,
                &self.game_map,
            );
            for pointik in fov {
                self.seen_tiles.insert(pointik);
            }
        }
    }

    pub fn generate_visible_ents_from_ent(&self, eid: &EntityID) -> Vec<EntityID> {
        let ent_pos = self.components.positions.get(eid).unwrap_or(&(0, 0));

        let mut visible_ents_with_self = self.game_map.generate_visible_ents_from_point(ent_pos);
        remove_ent_from_vec(&mut visible_ents_with_self, eid);

        visible_ents_with_self
    }

    pub fn line_from_ent_to_ent(
        &self,
        subj: &EntityID,
        obj: &EntityID,
    ) -> Option<BresenhamInclusive> {
        if let Some(startik) = self.components.positions.get(subj) {
            if let Some(endik) = self.components.positions.get(obj) {
                let start = Point::from_tuple(startik.clone());
                let end = Point::from_tuple(endik.clone());
                return Some(BresenhamInclusive::new(start, end));
            }
        }
        return None;
    }
    pub fn distance_from_ent_to_ent(
        &self,
        subj: &EntityID,
        obj: &EntityID,
    ) -> Option<CoordinateUnit> {
        if let Some(line) = self.line_from_ent_to_ent(subj, obj) {
            let mut distance = 0;
            for _ in line {
                distance += 1;
            }
            return Some(distance);
        } else {
            return None;
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn render_game_screen(&self, area: Rect, buf: &mut Buffer) {
        let client_pos = self
            .components
            .positions
            .get(&self.local_player_id)
            .unwrap_or(&(0, 0));

        let (_, selected_ent) = self.manage_item_vec_input();
        let highlighted_ranged_line =
            self.line_from_ent_to_ent(&self.local_player_id, &selected_ent);

        let client_graphics = self.game_map.create_client_render_packet_for_entity(
            &client_pos,
            &area,
            highlighted_ranged_line,
            &self.seen_tiles,
        );

        let mut render_lines = Vec::new();
        let needed_height = area.height as i16;

        if client_graphics.len() > 0 {
            for y in (0..needed_height) {
                let myspanvec: Vec<_> = client_graphics[y as usize]
                    .iter()
                    .map(|x| Span::from(x.0.clone()).fg(x.1).bg(x.2))
                    .collect();

                let myline = ratatui::text::Line::from(myspanvec);

                render_lines.push(myline);
            }
        }
        //neccesary beccause drawing is from the top
        render_lines.reverse();
        Paragraph::new(Text::from(render_lines))
            .on_black()
            .block(Block::new())
            .render(area, buf);
    }
    pub fn render_inventory_menus(&self, area: Rect, buf: &mut Buffer) {
        let mut inv_state = match self.selected_menu {
            ItemVecType::Inventory => self.item_list_state.clone(),
            _ => ListState::default(),
        };
        let mut equip_state = match self.selected_menu {
            ItemVecType::Equipped => self.item_list_state.clone(),
            _ => ListState::default(),
        };
        let mut ground_state = match self.selected_menu {
            ItemVecType::Ground => self.item_list_state.clone(),
            _ => ListState::default(),
        };
        let block = Block::bordered().title("Popup");
        let pop_area = popup_area(area, 80, 70);
        let pop_layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Min(20),
                Constraint::Min(20),
                Constraint::Min(20),
            ],
        )
        .split(pop_area);
        Clear.render(pop_area, buf); //this clears out the background
        block.render(pop_area, buf); //this clears out the background
        ratatui::prelude::StatefulWidget::render(
            self.render_item_list("Inventory", ItemVecType::Inventory),
            pop_layout[1],
            buf,
            &mut inv_state,
        );
        ratatui::prelude::StatefulWidget::render(
            self.render_item_list("Equipped", ItemVecType::Equipped),
            pop_layout[2],
            buf,
            &mut equip_state,
        );
        ratatui::prelude::StatefulWidget::render(
            self.render_item_list("Ground", ItemVecType::Ground),
            pop_layout[0],
            buf,
            &mut ground_state,
        );
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout_base = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(99), Constraint::Length(1)],
        )
        .split(area);
        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(150), Constraint::Length(30)],
        )
        .split(layout_base[0]);

        let left_layout = layout[0];
        let right_layout = layout[1];
        let key_hint_layout = layout_base[1];
        self.render_key_hints(key_hint_layout, buf);

        let second_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(80), Constraint::Min(10)],
        )
        .split(left_layout);

        let third_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(14), Constraint::Fill(1)],
        )
        .split(right_layout);

        let constant_info_layout = third_layout[0];

        let side_info_layout = third_layout[1];

        let game_screen_layout = second_layout[0];
        let event_layout = second_layout[1];

        let mut ranged_state = match self.input_state {
            InputState::RangedAttack => self.item_list_state.clone(),
            _ => ListState::default(),
        };

        self.render_game_screen(game_screen_layout, buf);

        self.render_const_info(constant_info_layout, buf);
        self.render_info_paragraph(side_info_layout, buf);
        self.render_event_paragraph(event_layout, buf);

        match self.input_state {
            InputState::Basic => (),

            InputState::Inventory => self.render_inventory_menus(game_screen_layout, buf),

            InputState::RangedAttack => {
                Clear.render(side_info_layout, buf); //this clears out the background

                ratatui::prelude::StatefulWidget::render(
                    self.render_ranged_attackable_list(),
                    side_info_layout,
                    buf,
                    &mut ranged_state,
                );
            }
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

pub fn remove_ent_from_vec(ent_vec: &mut Vec<EntityID>, ent_to_remove: &EntityID) {
    ent_vec.retain(|x| x != ent_to_remove);
}
