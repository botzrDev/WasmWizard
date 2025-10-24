/// Advertisement configuration and management
/// 
/// This module handles advertisement display, tracking, and monetization
use serde::{Deserialize, Serialize};

/// Advertisement placement locations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdPlacement {
    /// Header banner advertisement
    Header,
    /// Footer banner advertisement
    Footer,
    /// Sidebar advertisement
    Sidebar,
    /// In-content advertisement
    InContent,
    /// Mobile-specific advertisement
    Mobile,
}

/// Advertisement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advertisement {
    /// Unique identifier for the ad
    pub id: String,
    /// Ad placement location
    pub placement: AdPlacement,
    /// HTML content of the ad
    pub html_content: Option<String>,
    /// Google AdSense client ID (e.g., "ca-pub-XXXXXXXXXX")
    pub adsense_client: Option<String>,
    /// Google AdSense ad slot ID
    pub adsense_slot: Option<String>,
    /// Ad format (e.g., "auto", "horizontal", "vertical")
    pub ad_format: String,
    /// Whether the ad is currently active
    pub active: bool,
    /// Priority for ad selection (higher = more priority)
    pub priority: i32,
}

impl Advertisement {
    /// Create a new advertisement configuration
    pub fn new(id: String, placement: AdPlacement) -> Self {
        Self {
            id,
            placement,
            html_content: None,
            adsense_client: None,
            adsense_slot: None,
            ad_format: "auto".to_string(),
            active: true,
            priority: 0,
        }
    }

    /// Set Google AdSense configuration
    pub fn with_adsense(mut self, client: String, slot: String) -> Self {
        self.adsense_client = Some(client);
        self.adsense_slot = Some(slot);
        self
    }

    /// Set custom HTML content
    pub fn with_html(mut self, html: String) -> Self {
        self.html_content = Some(html);
        self
    }

    /// Set ad format
    pub fn with_format(mut self, format: String) -> Self {
        self.ad_format = format;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Render the advertisement HTML
    pub fn render(&self) -> String {
        if !self.active {
            return String::new();
        }

        // If custom HTML is provided, use it
        if let Some(ref html) = self.html_content {
            return html.clone();
        }

        // Otherwise, render Google AdSense
        if let (Some(ref client), Some(ref slot)) = (&self.adsense_client, &self.adsense_slot) {
            format!(
                r#"<ins class="adsbygoogle"
     style="display:block"
     data-ad-client="{}"
     data-ad-slot="{}"
     data-ad-format="{}"
     data-full-width-responsive="true"></ins>
<script>
     (adsbygoogle = window.adsbygoogle || []).push({{}});
</script>"#,
                client, slot, self.ad_format
            )
        } else {
            // Fallback: show a placeholder
            r#"<div class="ad-placeholder" style="background: #f0f0f0; padding: 2rem; text-align: center; border-radius: 8px; border: 2px dashed #ddd;">
    <p style="margin: 0; color: #666;">Advertisement Space Available</p>
    <p style="margin: 0.5rem 0 0; font-size: 0.875rem; color: #999;">Support us by disabling your ad blocker</p>
</div>"#.to_string()
        }
    }
}

/// Advertisement manager for handling multiple ads
#[derive(Debug, Clone)]
pub struct AdManager {
    ads: Vec<Advertisement>,
}

impl AdManager {
    /// Create a new ad manager
    pub fn new() -> Self {
        Self { ads: Vec::new() }
    }

    /// Add an advertisement to the manager
    pub fn add_ad(&mut self, ad: Advertisement) {
        self.ads.push(ad);
    }

    /// Get all active advertisements for a specific placement
    pub fn get_ads_for_placement(&self, placement: AdPlacement) -> Vec<&Advertisement> {
        let mut ads: Vec<&Advertisement> = self
            .ads
            .iter()
            .filter(|ad| ad.active && ad.placement == placement)
            .collect();
        
        // Sort by priority (higher priority first)
        ads.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        ads
    }

    /// Get the highest priority advertisement for a placement
    pub fn get_ad_for_placement(&self, placement: AdPlacement) -> Option<&Advertisement> {
        self.get_ads_for_placement(placement).first().copied()
    }

    /// Render advertisement HTML for a specific placement
    pub fn render_placement(&self, placement: AdPlacement) -> String {
        if let Some(ad) = self.get_ad_for_placement(placement) {
            ad.render()
        } else {
            String::new()
        }
    }
}

impl Default for AdManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advertisement_creation() {
        let ad = Advertisement::new("test-ad".to_string(), AdPlacement::Header)
            .with_adsense("ca-pub-123456".to_string(), "slot-789".to_string())
            .with_format("horizontal".to_string())
            .with_priority(10);

        assert_eq!(ad.id, "test-ad");
        assert_eq!(ad.placement, AdPlacement::Header);
        assert_eq!(ad.adsense_client, Some("ca-pub-123456".to_string()));
        assert_eq!(ad.priority, 10);
        assert!(ad.active);
    }

    #[test]
    fn test_ad_manager() {
        let mut manager = AdManager::new();
        
        let ad1 = Advertisement::new("ad1".to_string(), AdPlacement::Header)
            .with_priority(5);
        let ad2 = Advertisement::new("ad2".to_string(), AdPlacement::Header)
            .with_priority(10);
        
        manager.add_ad(ad1);
        manager.add_ad(ad2);
        
        let top_ad = manager.get_ad_for_placement(AdPlacement::Header).unwrap();
        assert_eq!(top_ad.id, "ad2"); // Higher priority
    }
}
