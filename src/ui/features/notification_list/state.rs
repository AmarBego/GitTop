#[derive(Debug, Clone)]
pub struct NotificationListState {
    pub scroll_offset: f32,
    pub viewport_height: f32,
}

impl Default for NotificationListState {
    fn default() -> Self {
        Self {
            scroll_offset: 0.0,
            viewport_height: 600.0, // Default fallback
        }
    }
}

impl NotificationListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.scroll_offset = 0.0;
    }

    pub fn update_viewport(&mut self, viewport: &iced::widget::scrollable::Viewport) {
        self.scroll_offset = viewport.absolute_offset().y;
        self.viewport_height = viewport.bounds().height;
    }

    /// Calculate the range of items to render for virtual scrolling.
    /// Returns (start_index, end_index) of flattened items.
    ///
    /// This is a helper for the view logic.
    pub fn calculate_visible_range(
        &self,
        item_height: f32,
        column_spacing: f32,
        buffer_items: usize,
        items_start_y: f32,
        total_items_count: usize,
    ) -> (usize, usize) {
        let first_visible_px = self.scroll_offset.max(0.0);
        let last_visible_px = self.scroll_offset + self.viewport_height + 100.0;
        let items_end_y = items_start_y
            + (total_items_count as f32 * (item_height + column_spacing) - column_spacing);

        if items_end_y < first_visible_px || items_start_y > last_visible_px {
            return (0, 0); // Not visible
        }

        let first_visible_idx = if first_visible_px > items_start_y {
            ((first_visible_px - items_start_y) / (item_height + column_spacing)) as usize
        } else {
            0
        };

        let last_visible_idx = if last_visible_px < items_end_y {
            ((last_visible_px - items_start_y) / (item_height + column_spacing)).ceil() as usize + 1
        } else {
            total_items_count
        };

        let render_start = first_visible_idx.saturating_sub(buffer_items);
        let render_end = (last_visible_idx + buffer_items).min(total_items_count);

        (render_start, render_end)
    }
}
