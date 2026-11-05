/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Gestor desacoplado de estado para la selección múltiple en la lista principal (paridad con `EstadoSeleccionLista` de Android).

use std::collections::HashSet;

use crate::models::Entrada;

#[derive(Debug, Default, Clone)]
pub struct EstadoSeleccionLista {
    pub modo_seleccion: bool,
    pub seleccionados: HashSet<String>,
}

impl EstadoSeleccionLista {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limpiar(&mut self) {
        self.salir_de_seleccion();
    }

    pub fn contiene(&self, id: &str) -> bool {
        self.seleccionados.contains(id)
    }

    pub fn conteo(&self) -> usize {
        self.seleccionados.len()
    }

    pub fn ids_seleccionados(&self) -> &HashSet<String> {
        &self.seleccionados
    }

    pub fn todas_seleccionadas(&self, visibles_ids: &[String]) -> bool {
        !visibles_ids.is_empty() && visibles_ids.iter().all(|id| self.seleccionados.contains(id))
    }

    pub fn seleccionar_todas(&mut self, visibles_ids: &[String]) {
        self.modo_seleccion = !visibles_ids.is_empty();
        self.seleccionados = visibles_ids.iter().cloned().collect();
    }

    pub fn alternar_id(&mut self, id: &str) {
        self.alternar_seleccion(id);
    }

    pub fn alternar_grupo(&mut self, ids: &[String]) {
        self.alternar_seleccion_lote(ids);
    }

    pub fn salir_de_seleccion(&mut self) {
        self.modo_seleccion = false;
        self.seleccionados.clear();
    }

    pub fn entrar_en_seleccion(&mut self, id: &str) {
        if id.is_empty() {
            return;
        }
        self.modo_seleccion = true;
        self.seleccionados.clear();
        self.seleccionados.insert(id.to_string());
    }

    pub fn alternar_seleccion(&mut self, id: &str) {
        if id.is_empty() {
            return;
        }
        if self.seleccionados.contains(id) {
            self.seleccionados.remove(id);
        } else {
            self.seleccionados.insert(id.to_string());
        }
        self.modo_seleccion = !self.seleccionados.is_empty();
    }

    pub fn alternar_seleccion_lote(&mut self, ids: &[String]) {
        if ids.is_empty() {
            return;
        }
        let todos_seleccionados = ids.iter().all(|id| self.seleccionados.contains(id));
        if todos_seleccionados {
            for id in ids {
                self.seleccionados.remove(id);
            }
        } else {
            for id in ids {
                self.seleccionados.insert(id.clone());
            }
        }
        self.modo_seleccion = !self.seleccionados.is_empty();
    }

    pub fn alternar_seleccionar_todo(&mut self, visibles_ids: &[String]) {
        let todo_seleccionado = !visibles_ids.is_empty()
            && visibles_ids.iter().all(|id| self.seleccionados.contains(id));
        if todo_seleccionado {
            self.seleccionados.clear();
            self.modo_seleccion = false;
        } else {
            self.modo_seleccion = !visibles_ids.is_empty();
            self.seleccionados = visibles_ids.iter().cloned().collect();
        }
    }

    pub fn sincronizar_con_entradas_vivas(&mut self, entradas: &[Entrada], en_papelera: bool) {
        if self.modo_seleccion {
            let vivos: HashSet<&str> = entradas
                .iter()
                .filter(|e| if en_papelera { e.eliminada_en > 0 } else { e.eliminada_en == 0 })
                .map(|e| e.id.as_str())
                .collect();
            self.seleccionados.retain(|id| vivos.contains(id.as_str()));
            if self.seleccionados.is_empty() {
                self.modo_seleccion = false;
            }
        }
    }
}
