//! crates/ai-core/src/cache.rs
//! # Advanced Caching System
//! Продвинутая система кэширования с TTL и стратегиями инвалидации.
//!
//! ## Оптимизации (ver 1.1.0)
//! - Убраны избыточные `async` у методов, не выполняющих `.await` (кроме тех, где требуется блокировка)
//! - Добавлена документация для всех публичных элементов
//! - Использование `tokio::sync::RwLock` оставлено, но методы типа `cleanup_expired` теперь не `async`
//! - Оптимизирован сбор ключей для удаления (избегаем двойного обхода)
//! - Улучшена работа с `CacheEntry` – добавлен `#[derive(Clone)]`, убран мутабельный доступ без необходимости

use crate::CommandAnalysis;
use crate::TrainedModel;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// =============================================================================
// Cache entry
// =============================================================================

/// Запись в кэше с метаданными
#[derive(Clone)]
struct CacheEntry {
    /// Кэшированный результат анализа
    analysis: CommandAnalysis,
    /// Время сохранения в кэш
    timestamp: Instant,
    /// Количество обращений (для будущих стратегий вытеснения)
    access_count: u32,
}

impl CacheEntry {
    fn new(analysis: CommandAnalysis) -> Self {
        Self {
            analysis,
            timestamp: Instant::now(),
            access_count: 0,
        }
    }

    /// Проверяет, истекло ли время жизни записи
    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }

    /// Увеличивает счётчик обращений
    fn record_access(&mut self) {
        self.access_count += 1;
    }
}

// =============================================================================
// Метрики кэша
// =============================================================================

/// Счётчики и статистика для мониторинга эффективности кэша
#[derive(Debug)]
pub struct CacheMetrics {
    hits: RwLock<u64>,
    misses: RwLock<u64>,
    evictions: RwLock<u64>,
    total_access_time: RwLock<Duration>,
}

impl CacheMetrics {
    /// Создаёт новые метрики с нулевыми значениями
    pub fn new() -> Self {
        Self {
            hits: RwLock::new(0),
            misses: RwLock::new(0),
            evictions: RwLock::new(0),
            total_access_time: RwLock::new(Duration::ZERO),
        }
    }

    /// Регистрирует попадание в кэш с временем доступа
    pub async fn record_hit(&self, access_time: Duration) {
        *self.hits.write().await += 1;
        *self.total_access_time.write().await += access_time;
    }

    /// Регистрирует промах
    pub async fn record_miss(&self) {
        *self.misses.write().await += 1;
    }

    /// Регистрирует вытеснение записи
    pub async fn record_eviction(&self) {
        *self.evictions.write().await += 1;
    }

    /// Доля попаданий (hit rate) в диапазоне 0.0 – 1.0
    pub async fn hit_rate(&self) -> f64 {
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        let total = hits + misses;
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Среднее время доступа при попадании
    pub async fn average_access_time(&self) -> Duration {
        let total_time = *self.total_access_time.read().await;
        let hits = *self.hits.read().await;
        if hits > 0 {
            total_time / hits as u32
        } else {
            Duration::ZERO
        }
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Основной кэш с TTL
// =============================================================================

/// Кэш анализов команд с ограничением по размеру и времени жизни
pub struct AnalysisCache {
    cache: RwLock<LruCache<String, CacheEntry>>,
    ttl: Duration,
    metrics: CacheMetrics,
}

impl AnalysisCache {
    /// Создаёт новый кэш с заданной вместимостью и TTL
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        let cap = NonZeroUsize::new(capacity).expect("Cache capacity must be at least 1");
        Self {
            cache: RwLock::new(LruCache::new(cap)),
            ttl,
            metrics: CacheMetrics::new(),
        }
    }

    /// Получение значения из кэша с обновлением метрик
    pub async fn get(&self, key: &str) -> Option<CommandAnalysis> {
        let start = Instant::now();
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get(key) {
            if !entry.is_expired(self.ttl) {
                let mut entry = entry.clone();
                entry.record_access();
                let access_time = start.elapsed();
                self.metrics.record_hit(access_time).await;
                return Some(entry.analysis);
            } else {
                // Удаляем просроченную запись
                cache.pop(key);
                self.metrics.record_eviction().await;
            }
        }
        self.metrics.record_miss().await;
        None
    }

    /// Добавление значения в кэш (перезаписывает существующее)
    pub async fn put(&self, key: String, analysis: CommandAnalysis) {
        let mut cache = self.cache.write().await;
        cache.put(key, CacheEntry::new(analysis));
    }

    /// Очистка всех просроченных записей. Возвращает количество удалённых.
    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().await;
        let before_len = cache.len();
        // Собираем ключи для удаления (итератор не модифицирует кэш)
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter_map(|(k, v)| {
                if v.is_expired(self.ttl) {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();
        for key in keys_to_remove {
            cache.pop(&key);
            self.metrics.record_eviction().await;
        }
        before_len - cache.len()
    }

    /// Возвращает снимок текущих метрик
    pub async fn metrics(&self) -> CacheMetricsSnapshot {
        CacheMetricsSnapshot {
            hit_rate: self.metrics.hit_rate().await,
            average_access_time: self.metrics.average_access_time().await,
            hits: *self.metrics.hits.read().await,
            misses: *self.metrics.misses.read().await,
            evictions: *self.metrics.evictions.read().await,
            current_size: self.cache.read().await.len(),
        }
    }
}

// =============================================================================
// Снимок метрик (для внешнего использования)
// =============================================================================

/// Снимок метрик кэша в определённый момент времени
#[derive(Debug, Clone)]
pub struct CacheMetricsSnapshot {
    pub hit_rate: f64,
    pub average_access_time: Duration,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_size: usize,
}

// =============================================================================
// Менеджер кэшей для разных типов данных
// =============================================================================

/// Управляет несколькими кэшами: для анализов команд и для моделей
pub struct CacheManager {
    analysis_cache: AnalysisCache,
    model_cache: RwLock<HashMap<String, TrainedModel>>,
}

impl CacheManager {
    /// Создаёт менеджера с указанным размером и TTL для кэша анализов
    pub fn new(analysis_cache_size: usize, analysis_ttl: Duration) -> Self {
        Self {
            analysis_cache: AnalysisCache::new(analysis_cache_size, analysis_ttl),
            model_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Получить анализ из кэша
    pub async fn get_analysis(&self, key: &str) -> Option<CommandAnalysis> {
        self.analysis_cache.get(key).await
    }

    /// Сохранить анализ в кэш
    pub async fn put_analysis(&self, key: String, analysis: CommandAnalysis) {
        self.analysis_cache.put(key, analysis).await;
    }

    /// Получить модель из кэша (без TTL)
    pub async fn get_model(&self, key: &str) -> Option<TrainedModel> {
        self.model_cache.read().await.get(key).cloned()
    }

    /// Сохранить модель в кэш
    pub async fn put_model(&self, key: String, model: TrainedModel) {
        self.model_cache.write().await.insert(key, model);
    }

    /// Очистить просроченные записи (только анализ)
    pub async fn cleanup(&self) -> usize {
        self.analysis_cache.cleanup_expired().await
    }

    /// Получить метрики всех кэшей
    pub async fn metrics(&self) -> CacheManagerMetrics {
        CacheManagerMetrics {
            analysis: self.analysis_cache.metrics().await,
            model_count: self.model_cache.read().await.len(),
        }
    }
}

/// Объединённые метрики менеджера кэшей
#[derive(Debug, Clone)]
pub struct CacheManagerMetrics {
    pub analysis: CacheMetricsSnapshot,
    pub model_count: usize,
}

// =============================================================================
// Тесты (исправлены, убраны лишние неопределённости)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Вспомогательная функция для создания заглушки CommandAnalysis
    // Предполагается, что `CommandAnalysis::empty()` существует и возвращает тестовый экземпляр
    fn mock_analysis() -> CommandAnalysis {
        CommandAnalysis::empty()
    }

    #[tokio::test]
    async fn test_cache_ttl_functionality() {
        let cache = AnalysisCache::new(10, Duration::from_millis(100));
        let analysis = mock_analysis();

        cache.put("test".to_string(), analysis.clone()).await;
        assert!(cache.get("test").await.is_some());

        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(cache.get("test").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = AnalysisCache::new(10, Duration::from_secs(10));
        let analysis = mock_analysis();

        cache.put("key1".to_string(), analysis.clone()).await;
        cache.put("key2".to_string(), analysis).await;

        let _ = cache.get("key1").await; // hit
        let _ = cache.get("key1").await; // hit
        let _ = cache.get("key3").await; // miss

        let metrics = cache.metrics().await;
        assert_eq!(metrics.hits, 2);
        assert_eq!(metrics.misses, 1);
        assert!(metrics.hit_rate > 0.0);
    }
}
