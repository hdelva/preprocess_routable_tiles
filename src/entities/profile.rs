#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use crate::entities::way::Way;
use crate::entities::node::Node;
use crate::util::haversine::haversine;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    hasMaxSpeed: Option<f64>,
    hasAccessRules: Vec<Rule>,
    hasOnewayRules: Vec<Rule>,
    hasSpeedRules: Vec<Rule>,
    hasPriorityRules: Vec<Rule>,
    hasObstacleRules: Vec<Rule>,
    hasObstacleTimeRules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    r#match: Option<Condition>,
    concludes: Conclusion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Condition {
    hasPredicate: Option<String>,
    hasObject: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conclusion {
    hasAccess: Option<bool>,
    isOneway: Option<bool>,
    isReversed: Option<bool>,
    hasSpeed: Option<f64>,
    isObstacle: Option<bool>,
    hasObstacleTime: Option<f64>,
    hasPriority: Option<f64>,
}

impl Profile {
    pub fn get_used_concepts(&self) -> HashSet<String> {
        let mut result = HashSet::new();

        let mut extract = |rules: &[Rule]| {
            for rule in rules.iter() {
                if let Some(Condition {hasPredicate, hasObject}) = &rule.r#match {
                    if let Some(uri) = hasPredicate {
                        result.insert(uri.to_string());
                    }
                    if let Some(uri) = hasObject {
                        result.insert(uri.to_string());
                    }
                }
            }
        };

        extract(&self.hasAccessRules);
        extract(&self.hasOnewayRules);
        extract(&self.hasSpeedRules);
        extract(&self.hasPriorityRules);
        extract(&self.hasObstacleRules);
        extract(&self.hasObstacleTimeRules);
        
        return result;
    }

    pub fn is_one_way(&self, way: &Way) -> bool {
        for rule in &self.hasOnewayRules {
            let conclusion = rule.concludes.isOneway.unwrap();
            if let Some(ref condition) = rule.r#match {
                if let Condition {
                    hasObject: Some(ref value),
                    hasPredicate: Some(ref key)
                } = condition {
                    if way.get_tags().contains_key(key) && &way.get_tags()[key] == value {
                        return conclusion
                    }
                }
            } else {
                return conclusion;
            }
        }

        return false;
    }

    pub fn has_access(&self, way: &Way) -> bool {
        for rule in &self.hasAccessRules {
            let conclusion = rule.concludes.hasAccess.unwrap();
            if let Some(ref condition) = rule.r#match {
                if let Condition {
                    hasObject: Some(ref value),
                    hasPredicate: Some(ref key)
                } = condition {
                    if way.get_tags().contains_key(key) && &way.get_tags()[key] == value {
                        return conclusion
                    }
                }
            } else {
                return conclusion;
            }
        }

        return true;
    }

    pub fn get_default_speed(&self) -> f64 {
        return 5.;
    }

    pub fn get_max_speed(&self) -> f64 {
        self.hasMaxSpeed.unwrap_or(300.)
    }

    pub fn get_speed(&self, way: &Way) -> f64 {
        let speed_limit = way
            .get_max_speed()
            .unwrap_or(std::f64::MAX)
            .min(self.get_max_speed());

        for rule in &self.hasSpeedRules {
            let conclusion = rule.concludes.hasSpeed.unwrap();
            if let Some(ref condition) = rule.r#match {
                if let Condition {
                    hasObject: Some(ref value),
                    hasPredicate: Some(ref key)
                } = condition {
                    if way.get_tags().contains_key(key) && &way.get_tags()[key] == value {
                        return conclusion.min(speed_limit);
                    }
                }
            } else {
                return conclusion.min(speed_limit);
            }
        }

        return speed_limit.min(self.get_default_speed());
    }

    fn get_distance(&self, from: &Node, to: &Node) -> f64 {
        // distance in km
        haversine(from.get_lat(), to.get_lat(),from.get_long(), to.get_long())
    }

    pub fn get_duration(&self, from: &Node, to: &Node, way: &Way) -> f64 {
        let distance = self.get_distance(from, to);
        let speed = self.get_speed(way);
        let time = distance / speed; // h
        return time * 60. *60. * 1000.; // ms
    }

    pub fn get_multiplier(&self, way: &Way) -> f64 {
        for rule in &self.hasPriorityRules {
            let conclusion = rule.concludes.hasPriority.unwrap();
            if let Some(ref condition) = rule.r#match {
                if let Condition {
                    hasObject: Some(ref value),
                    hasPredicate: Some(ref key)
                } = condition {
                    if way.get_tags().contains_key(key) && &way.get_tags()[key] == value {
                        return 1. / conclusion;
                    }
                }
            } else {
                return 1. / conclusion;
            }
        }

        return 1.;
    }

    pub fn get_cost(&self, from: &Node, to: &Node, way: &Way) -> f64 {
        let base = self.get_multiplier(way) *
            (self.get_duration(from, to, way) + self.get_obstacle_time(to));
        base.max(1.)
    }

    pub fn is_obstacle(&self, node: &Node) -> bool {
        for rule in &self.hasObstacleRules {
            let conclusion = rule.concludes.isObstacle.unwrap();
            if let Some(ref condition) = rule.r#match {
                if let Condition {
                    hasObject: Some(ref value),
                    hasPredicate: Some(ref key)
                } = condition {
                    if node.get_tags().contains_key(key) && &node.get_tags()[key] == value {
                        return conclusion
                    }
                }
            } else {
                return conclusion;
            }
        }

        return false;
    }

    pub fn get_obstacle_time(&self, node: &Node) -> f64 {
        for rule in &self.hasObstacleTimeRules {
            let conclusion = rule.concludes.hasObstacleTime.unwrap();
            if let Some(ref condition) = rule.r#match {
                if let Condition {
                    hasObject: Some(ref value),
                    hasPredicate: Some(ref key)
                } = condition {
                    if node.get_tags().contains_key(key) && &node.get_tags()[key] == value {
                        return conclusion * 1000.;
                    }
                }
            } else {
                return conclusion * 1000.;
            }
        }

        return 0.;
    }
}