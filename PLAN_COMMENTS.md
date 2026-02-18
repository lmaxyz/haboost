# План: Чтение комментариев к статье

## Статус: ✅ Реализовано

## Цель
Добавить возможность чтения комментариев к статье. В конце статьи должна быть кнопка "Комментарии", открывающая окно с комментариями.

## API
Endpoint: `https://habr.com/kek/v2/articles/{article_id}/comments/split/{page}?fl=ru&hl=ru`
- Пример: `https://habr.com/kek/v2/articles/988672/comments/split/guest?fl=ru&hl=ru`
- Пагинация через `{page}` (guest, 1, 2, ...)

## Реализованные этапы

### 1. ✅ Создание структур данных
- Создан `src/habr_client/comment.rs` с:
  - `Comment` — структура комментария (автор, текст, дата, рейтинг, вложенные комментарии)
  - `CommentAuthor` — автор комментария
  - `CommentsResponse` — ответ API

### 2. ✅ Добавление API-метода
- Добавлен `HabrClient::get_comments(article_id, page)` в `src/habr_client/mod.rs`
- Поддержка пагинации через параметр page

### 3. ✅ UI-компонент
- Создан `src/views/comments.rs`
- Реализует трейт `UiView`
- Отображение списка комментариев с вложенностью
- Пагинация (кнопки "Назад"/"Далее")

### 4. ✅ Интеграция в ArticleDetails
- Добавлена кнопка "Комментарии (N)" в конце статьи
- При нажатии открывается окно комментариев через ViewStack
- Поле `comments_count` добавлено в `ArticleData`

## Файлы изменения
- Создан: `src/habr_client/comment.rs`
- Изменён: `src/habr_client/mod.rs` — импорт и метод
- Создан: `src/views/comments.rs`
- Изменён: `src/views/article_details.rs` — кнопка
- Изменён: `src/views/mod.rs` — экспорт
- Изменён: `src/habr_client/article.rs` — поле comments_count
