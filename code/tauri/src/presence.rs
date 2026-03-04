use chrono::{Local, Timelike};
use serde::Deserialize;
use std::hash::{Hash, Hasher};
use tauri::Emitter;

const PERSONAS_JSON: &str = include_str!("../../frontend/assets/personas.json");

// ── Data types ──────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Persona {
    id: String,
    name: String,
    active_hours: [u32; 2],
    session_minutes: u32,
    message_frequency: String,
    messages: Messages,
}

#[derive(Deserialize)]
struct Messages {
    enter: Vec<String>,
    exit: Vec<String>,
    during: Vec<String>,
    encourage: Vec<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct PresenceMessage {
    pub name: String,
    pub message: String,
}

struct Event {
    at: u32, // seconds since midnight (0..86400)
    msg: PresenceMessage,
}

// ── Simple xorshift64 PRNG ──────────────────────────────────

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn range(&mut self, n: u32) -> u32 {
        (self.next() % n as u64) as u32
    }

    fn chance(&mut self, p: f64) -> bool {
        (self.next() % 10000) < (p * 10000.0) as u64
    }

    fn pick<'a>(&mut self, items: &'a [String]) -> &'a str {
        &items[self.range(items.len() as u32) as usize]
    }
}

fn seed_for(date: &str, id: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    date.hash(&mut h);
    id.hash(&mut h);
    h.finish()
}

// ── Schedule generation ─────────────────────────────────────

fn build_schedule(personas: &[Persona], date: &str) -> Vec<Event> {
    let mut events = Vec::new();

    for p in personas {
        let mut rng = Rng::new(seed_for(date, &p.id));

        // Skip some personas each day based on frequency
        let prob = match p.message_frequency.as_str() {
            "high" => 0.85,
            "medium" => 0.65,
            _ => 0.45,
        };
        if !rng.chance(prob) {
            continue;
        }

        let (sh, eh) = (p.active_hours[0] % 24, p.active_hours[1] % 24);

        // For wrap-around hours (e.g. 22→3), pick evening or morning portion
        let (eff_start, eff_end) = if sh < eh {
            (sh, eh)
        } else if rng.chance(0.6) {
            (sh, 24) // evening portion (slightly more likely)
        } else {
            (0, eh) // morning portion
        };

        let window_min = (eff_end - eff_start) * 60;
        if window_min == 0 {
            continue;
        }
        let max_offset = window_min.saturating_sub(p.session_minutes).max(1);
        let offset = rng.range(max_offset);

        let start_min = eff_start * 60 + offset;
        let variance = rng.range(11); // 0..10
        let duration = p.session_minutes.saturating_sub(5) + variance; // ±5 min
        let end_min = (start_min + duration).min(eff_end * 60);

        // Enter message
        if !p.messages.enter.is_empty() {
            events.push(Event {
                at: start_min * 60,
                msg: PresenceMessage {
                    name: p.name.clone(),
                    message: rng.pick(&p.messages.enter).into(),
                },
            });
        }

        // During messages
        let n_during = match p.message_frequency.as_str() {
            "high" => 1 + rng.range(2), // 1-2
            "medium" => rng.range(2),    // 0-1
            _ => {
                if rng.chance(0.3) {
                    1
                } else {
                    0
                }
            }
        };
        if !p.messages.during.is_empty() {
            for i in 0..n_during {
                let frac = (i + 1) as f64 / (n_during + 1) as f64;
                let t_min = start_min + (duration as f64 * frac) as u32;
                let jitter_sec = rng.range(120); // 0..2 min jitter
                let at = (t_min * 60 + jitter_sec).min(end_min * 60);
                events.push(Event {
                    at,
                    msg: PresenceMessage {
                        name: p.name.clone(),
                        message: rng.pick(&p.messages.during).into(),
                    },
                });
            }
        }

        // Exit message
        if !p.messages.exit.is_empty() {
            events.push(Event {
                at: end_min * 60,
                msg: PresenceMessage {
                    name: p.name.clone(),
                    message: rng.pick(&p.messages.exit).into(),
                },
            });
        }

        // Encourage (20% chance, during the session)
        if !p.messages.encourage.is_empty() && rng.chance(0.2) && duration > 0 {
            let t_min = start_min + rng.range(duration);
            events.push(Event {
                at: t_min * 60,
                msg: PresenceMessage {
                    name: p.name.clone(),
                    message: rng.pick(&p.messages.encourage).into(),
                },
            });
        }
    }

    events.sort_by_key(|e| e.at);
    events
}

// ── Scheduler loop ──────────────────────────────────────────

pub fn spawn(app: tauri::AppHandle) {
    let personas: Vec<Persona> =
        serde_json::from_str(PERSONAS_JSON).expect("failed to parse personas.json");

    tauri::async_runtime::spawn(async move {
        let mut date = String::new();
        let mut schedule: Vec<Event> = Vec::new();
        let mut cursor: usize = 0;

        let mut tick = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            tick.tick().await;
            let now = Local::now();
            let today = now.format("%Y-%m-%d").to_string();
            let now_secs = now.hour() * 3600 + now.minute() * 60 + now.second();

            // Regenerate schedule on new day
            if today != date {
                schedule = build_schedule(&personas, &today);
                date = today;
                cursor = schedule
                    .iter()
                    .position(|e| e.at > now_secs)
                    .unwrap_or(schedule.len());
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] presence: {} events scheduled", schedule.len());
                }
            }

            // Emit events whose time has passed
            while cursor < schedule.len() && schedule[cursor].at <= now_secs {
                let ev = &schedule[cursor];
                if cfg!(debug_assertions) {
                    let h = ev.at / 3600;
                    let m = (ev.at % 3600) / 60;
                    eprintln!(
                        "[52Hz] presence {:02}:{:02} {} — {}",
                        h, m, ev.msg.name, ev.msg.message
                    );
                }
                let _ = app.emit("presence-message", ev.msg.clone());
                cursor += 1;
            }
        }
    });
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personas_json_parses() {
        let personas: Vec<Persona> =
            serde_json::from_str(PERSONAS_JSON).expect("should parse personas.json");
        assert!(personas.len() >= 50);
    }

    #[test]
    fn schedule_generates_events() {
        let personas: Vec<Persona> = serde_json::from_str(PERSONAS_JSON).unwrap();
        let schedule = build_schedule(&personas, "2026-03-04");
        // With 50 personas, expect at least some events
        assert!(schedule.len() > 20, "got {} events", schedule.len());
        // Events should be sorted
        for w in schedule.windows(2) {
            assert!(w[0].at <= w[1].at);
        }
    }

    #[test]
    fn schedule_is_deterministic() {
        let personas: Vec<Persona> = serde_json::from_str(PERSONAS_JSON).unwrap();
        let s1 = build_schedule(&personas, "2026-03-04");
        let s2 = build_schedule(&personas, "2026-03-04");
        assert_eq!(s1.len(), s2.len());
        for (a, b) in s1.iter().zip(s2.iter()) {
            assert_eq!(a.at, b.at);
            assert_eq!(a.msg.name, b.msg.name);
            assert_eq!(a.msg.message, b.msg.message);
        }
    }

    #[test]
    fn different_days_give_different_schedules() {
        let personas: Vec<Persona> = serde_json::from_str(PERSONAS_JSON).unwrap();
        let s1 = build_schedule(&personas, "2026-03-04");
        let s2 = build_schedule(&personas, "2026-03-05");
        // Very unlikely to be identical
        let same = s1
            .iter()
            .zip(s2.iter())
            .filter(|(a, b)| a.at == b.at && a.msg.name == b.msg.name)
            .count();
        assert!(same < s1.len());
    }

    #[test]
    fn events_within_valid_time_range() {
        let personas: Vec<Persona> = serde_json::from_str(PERSONAS_JSON).unwrap();
        let schedule = build_schedule(&personas, "2026-03-04");
        for ev in &schedule {
            assert!(ev.at < 86400, "event at {} exceeds 24h", ev.at);
        }
    }
}
