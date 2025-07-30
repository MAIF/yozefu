use itertools::Itertools;

#[derive(Default)]
pub(crate) struct TopicList {
    topics: Vec<String>,
    selected: Vec<String>,
    filter: String,
}

impl TopicList {
    pub fn new(topics: Vec<String>, selected: Vec<String>) -> Self {
        let filtered_topics = topics
            .into_iter()
            .filter(|t| !selected.contains(t))
            .collect::<Vec<_>>();

        Self {
            topics: filtered_topics,
            selected: selected.into_iter().collect(),
            filter: String::new(),
        }
    }

    pub fn selected(&self) -> &[String] {
        &self.selected
    }

    pub fn any_selected(&self) -> bool {
        !&self.selected.is_empty()
    }

    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_string();
    }

    pub fn get_with_selection(&self) -> Vec<(&String, bool)> {
        let mut list = self.selected().iter().map(|t| (t, true)).collect_vec();
        match self.filter.is_empty() {
            true => {
                list.extend(self.topics.iter().map(|t| (t, self.selected.contains(t))));
            }
            false => {
                list.extend(
                    self.topics
                        .iter()
                        .filter(|t| t.contains(&self.filter))
                        .map(|t| (t, self.selected.contains(t))),
                );
            }
        }

        list
    }

    pub fn all(&self) -> &[String] {
        &self.topics
    }

    pub fn get(&self) -> Vec<&String> {
        self.get_with_selection()
            .iter()
            .map(|(t, _)| *t)
            .collect::<Vec<_>>()
    }

    pub fn refresh_topics(&mut self, new_topics: Vec<String>) {
        (self.topics, self.selected) = Self::init(&self.selected, new_topics);
    }

    fn init(selected: &[String], mut new_topics: Vec<String>) -> (Vec<String>, Vec<String>) {
        new_topics.dedup();
        new_topics.sort();

        let selected = new_topics
            .iter()
            .filter(|t| selected.contains(t))
            .cloned()
            .collect::<Vec<_>>();
        let topics = new_topics
            .into_iter()
            .filter(|t| !selected.contains(t))
            .collect();
        (topics, selected)
    }

    pub fn clear_selected(&mut self) {
        self.topics.extend(self.selected.iter().cloned());
        self.topics.dedup();
        self.topics.sort();
        self.selected.clear();
    }

    pub fn toggle_topics(&mut self, topic: &str) {
        if self.selected.contains(&topic.to_string()) {
            self.selected.retain(|t| t != topic);
            self.topics.push(topic.to_string());
        } else {
            self.selected.push(topic.to_string());
            self.topics.retain(|e| e != topic);
        }

        self.topics.sort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_topic_list() {
        let topics = vec!["ketchup".to_string(), "mayo".to_string()];
        let selected = vec!["ketchup".to_string()];
        let topic_list = TopicList::new(topics.clone(), selected.clone());
        assert_eq!(topic_list.selected(), vec!["ketchup".to_string()]);
        assert_eq!(
            topic_list.get_with_selection(),
            vec![(&"ketchup".to_string(), true), (&"mayo".to_string(), false)]
        );
    }

    #[test]
    fn test_refresh_topics() {
        let topics = vec!["1".to_string()];
        let selected = vec!["1".to_string()];
        let mut topic_list = TopicList::new(topics.clone(), selected.clone());

        topic_list.refresh_topics(vec!["1".to_string(), "3".to_string()]);
        assert_eq!(
            topic_list.get_with_selection(),
            vec![(&"1".to_string(), true), (&"3".to_string(), false)]
        );
    }
}
