# CHUẨN TRÌNH SOẠN THẢO NỘI DUNG ĐỊNH DẠNG (RICH TEXT EDITOR) - v2.0.0

> **Bắt buộc** với mọi module có chức năng soạn thảo nội dung định dạng trong hệ
> sinh thái tsudev: CMS, trang quản trị, mô tả sản phẩm, thông báo, bài viết.
>
> Mục tiêu: trải nghiệm soạn thảo **ngang Microsoft Word** - đầy đủ, chuyên
> nghiệp, không thiếu tính năng cơ bản - đồng thời an toàn trước XSS và tối ưu
> cho tiếng Việt.
>
> Ký hiệu: `MUST` / `MUST NOT` = bắt buộc, không đạt thì không được merge.
> `SHOULD` = nên, bỏ qua phải ghi lý do vào ADR hoặc PR. `MAY` = tùy chọn.

## Mục lục

- [1. Phạm vi](#1-phạm-vi)
- [2. Định hướng kỹ thuật](#2-định-hướng-kỹ-thuật)
- [3. Bộ tính năng định dạng bắt buộc](#3-bộ-tính-năng-định-dạng-bắt-buộc)
- [4. Bảng biểu](#4-bảng-biểu)
- [5. Tự động lưu và dán thông minh](#5-tự-động-lưu-và-dán-thông-minh)
- [6. Xử lý hình ảnh và tệp](#6-xử-lý-hình-ảnh-và-tệp)
- [7. Bảo mật](#7-bảo-mật)
- [8. Kiến trúc dữ liệu và đầu ra](#8-kiến-trúc-dữ-liệu-và-đầu-ra)
- [9. Cấu trúc thư mục và đặt tên](#9-cấu-trúc-thư-mục-và-đặt-tên)
- [10. Khả năng truy cập và responsive](#10-khả-năng-truy-cập-và-responsive)
- [11. Tiêu chí nghiệm thu](#11-tiêu-chí-nghiệm-thu)
- [12. Yêu cầu kiểm thử](#12-yêu-cầu-kiểm-thử)
- [13. Tài liệu tham khảo](#13-tài-liệu-tham-khảo)

---

## 1. Phạm vi

**Áp dụng cho** mọi module:

- Soạn thảo hoặc chỉnh sửa nội dung định dạng (bài viết, tin tức, mô tả, thông báo).
- Lưu nội dung định dạng dạng HTML hoặc JSON vào cơ sở dữ liệu.
- Hiển thị lại nội dung đó ra trang công khai.

**Không thuộc phạm vi:** xây dựng engine soạn thảo từ đầu; quy định chi tiết
giao diện trang quản trị nằm ngoài vùng editor.

Chức năng tìm kiếm và lọc nội dung do editor sinh ra được quy định riêng tại
[`SEARCH_AND_FILTER.md`](SEARCH_AND_FILTER.md).

---

## 2. Định hướng kỹ thuật

### 2.1. Nguyên tắc

- `MUST NOT` tự xây dựng engine soạn thảo mới từ đầu. Việc này tốn thời gian, dễ
  sinh lỗi khác biệt giữa các trình duyệt, và gần như chắc chắn tạo ra lỗ hổng
  XSS mà đội không đủ nguồn lực để rà soát.
- `MUST` tích hợp và tùy biến một thư viện mã nguồn mở đã được kiểm chứng.
- `MUST` cấu hình thư viện chạy ở chế độ **tự vận hành (self-hosted)**, không phụ
  thuộc dịch vụ đám mây trả phí của bên thứ ba. Đây là ràng buộc 0 đồng tại
  [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md).

### 2.2. Thư viện được duyệt

| Thư viện | Khi nên chọn | Mức | Ghi chú |
| --- | --- | --- | --- |
| **Tiptap** (trên nền ProseMirror) | Stack React/Vue hiện đại, cần tùy biến sâu, đầu ra JSON có cấu trúc | ⭐ | Ưu tiên số 1. Kiến trúc theo component, dễ mở rộng node/mark riêng |
| **CKEditor 5** | Cần bộ tính năng sẵn có gần Word nhất, ít thời gian tùy biến | ✅ | Bản mã nguồn mở (GPL) đủ cho phần lớn tính năng ở mục 3 |
| **TinyMCE** | Hệ thống cũ dùng jQuery/PHP truyền thống, cần tích hợp nhanh | ✅ | `MUST` cấu hình tự vận hành, không dùng khóa API đám mây |
| **Quill.js** | Cần editor nhẹ, tải nhanh, ít tính năng nâng cao | ⚠️ | Chỉ chọn khi **không** cần bảng biểu phức tạp hoặc gộp ô |
| **Lexical** | Cần hiệu năng cao trên tài liệu rất dài | ✅ | Hệ sinh thái phần mở rộng còn mỏng hơn Tiptap |

Lựa chọn cụ thể của từng project `MUST` được ghi vào ADR
(`docs/templates/ADR.md`) và tham chiếu ngược lại tài liệu này.

### 2.3. Tương thích

- `MUST` chạy đầy đủ tính năng trên **desktop**: Chrome, Firefox, Edge, Safari -
  hai phiên bản gần nhất.
- `MUST` chạy tốt trên **di động và máy tính bảng**: thanh công cụ rút gọn thông
  minh, hỗ trợ thao tác chạm, không vỡ bố cục.
- `MUST` gõ tiếng Việt có dấu mượt mà với mọi bộ gõ phổ biến (Unikey, EVKey,
  Gboard tiếng Việt, bộ gõ iOS): **không mất dấu, không nhảy con trỏ khi gõ
  nhanh**. Đây là lỗi phổ biến nhất khi tích hợp editor cho người dùng Việt Nam,
  do editor xử lý sự kiện `composition` sai. `MUST` kiểm thử thủ công với ít
  nhất hai bộ gõ khác nhau trước khi nghiệm thu.

---

## 3. Bộ tính năng định dạng bắt buộc

Ký hiệu mức: `M` = MUST, `S` = SHOULD, `O` = MAY.

### 3.1. Định dạng ký tự

| Tính năng | Mức | Ghi chú |
| --- | --- | --- |
| Đậm, Nghiêng, Gạch chân, Gạch ngang giữa | `M` | Phím tắt chuẩn `Ctrl/Cmd+B`, `+I`, `+U` |
| Chỉ số trên / chỉ số dưới | `S` | Cần cho nội dung kỹ thuật, công thức |
| Màu chữ | `M` | Bảng màu định sẵn theo token thương hiệu + bộ chọn màu tùy ý |
| Màu nền chữ (tô sáng) | `M` | Tối thiểu 6 màu tô sáng định sẵn |
| Xóa định dạng | `M` | Trả đoạn đang chọn về văn bản thuần, giữ nguyên cấu trúc đoạn |
| Cỡ chữ | `S` | `MUST` chọn từ thang cỡ chữ của `DESIGN_SYSTEM.md`, không nhập tự do |
| Họ phông chữ | `O` | Chỉ khi nghiệp vụ thật sự cần; mặc định dùng phông của hệ thống |

### 3.2. Cấu trúc nội dung

| Tính năng | Mức | Ghi chú |
| --- | --- | --- |
| Tiêu đề H1 đến H4, Đoạn văn | `M` | H1 chỉ một lần mỗi bài - ràng buộc ở tầng kiểm tra, phục vụ SEO và `ACCESSIBILITY.md` mục 4 |
| Danh sách chấm đầu dòng, danh sách đánh số | `M` | Hỗ trợ lồng nhau tối thiểu 3 cấp |
| Trích dẫn (blockquote) | `M` | |
| Đường phân cách ngang | `M` | |
| Danh sách việc cần làm (ô đánh dấu) | `O` | |
| Chú thích cuối trang | `O` | |

### 3.3. Căn chỉnh và trình bày

| Tính năng | Mức | Ghi chú |
| --- | --- | --- |
| Căn trái / giữa / phải / đều hai bên | `M` | |
| Thụt lề và bỏ thụt lề | `S` | |
| Giãn dòng | `O` | `MUST` chọn từ token `line-height`, không nhập tự do |
| Giãn ký tự | `O` | |

### 3.4. Chèn phương tiện

| Nhóm | Yêu cầu | Mức |
| --- | --- | --- |
| **Liên kết** | Chèn, sửa, xóa liên kết; tùy chọn mở tab mới với `target="_blank" rel="noopener noreferrer"`; `MUST` kiểm tra URL hợp lệ và chặn giao thức `javascript:` trước khi lưu | `M` |
| **Hình ảnh** | Tải trực tiếp từ máy; dán URL; kéo thả; dán ảnh từ bộ nhớ tạm; căn lề ảnh (trái/giữa/phải/tràn chiều rộng); **văn bản thay thế bắt buộc**; chú thích ảnh | `M` |
| **Video** | Nhúng từ YouTube/Vimeo qua liên kết hoặc iframe; tự động co giãn theo tỉ lệ khung hình | `M` |
| **Bảng** | Xem mục 4 | `M` |
| **Tệp đính kèm** | Tải lên và chèn liên kết tệp (PDF, DOCX...) | `O` |
| **Biểu tượng cảm xúc** | Bộ chọn có tìm kiếm | `O` |

### 3.5. Nâng cao và tiện ích

| Tính năng | Mức | Ghi chú |
| --- | --- | --- |
| Khối mã có tô màu cú pháp | `M` | Tối thiểu: JavaScript, TypeScript, Python, HTML, CSS, JSON, SQL, Bash |
| Hoàn tác / Làm lại | `M` | `Ctrl/Cmd+Z`, `Ctrl/Cmd+Shift+Z`. Lịch sử tối thiểu 50 bước |
| Đếm số từ và số ký tự theo thời gian thực | `M` | Đếm đúng quy tắc tách từ tiếng Việt - xem `SEARCH_AND_FILTER.md` mục 3 |
| Xem và sửa mã nguồn HTML | `M` | **Chỉ hiển thị cho vai trò Editor/Admin trở lên**, kèm cảnh báo rủi ro. Xem mục 7 |
| Ký tự đặc biệt | `S` | |
| Chế độ toàn màn hình | `S` | |
| Thanh công cụ dính khi cuộn trang dài | `S` | |
| Tìm và thay thế trong tài liệu | `S` | `MUST` hỗ trợ tìm không dấu như quy định tại `SEARCH_AND_FILTER.md` mục 4 |
| Mục lục tự động sinh từ tiêu đề | `O` | |
| Ghi chú và bình luận trên đoạn | `O` | |
| Theo dõi thay đổi | `O` | Chỉ khi nghiệp vụ biên tập nhiều người thật sự cần |

---

## 4. Bảng biểu

Bảng là phần người dùng quen với Word hay hụt hẫng nhất, nên tách riêng.

| Tính năng | Mức |
| --- | --- |
| Chèn bảng với số hàng/cột tùy chọn | `M` |
| Thêm và xóa hàng, cột | `M` |
| Gộp ô và tách ô | `M` |
| Căn chỉnh nội dung trong ô (ngang và dọc) | `M` |
| Hàng tiêu đề, giữ cố định khi cuộn | `M` |
| Đổi độ rộng cột bằng kéo thả | `S` |
| Màu nền ô và viền ô | `S` |
| Bảng lồng trong ô | `O` |

- Bảng `MUST` cuộn ngang được trong khung riêng trên màn hình hẹp. `MUST NOT` để
  bảng làm cả trang cuộn ngang.
- Bảng `MUST` sinh ra HTML có `<thead>` và `<th>` đúng ngữ nghĩa, không phải
  `<td>` in đậm - trình đọc màn hình dựa vào đó để đọc bảng.

---

## 5. Tự động lưu và dán thông minh

### 5.1. Tự động lưu

- `MUST` tự động lưu bản nháp mỗi **30 đến 60 giây**, và lưu thêm khi người dùng
  rời tab hoặc rời trang (`beforeunload`) nếu còn thay đổi chưa lưu.
- `MUST` hiển thị trạng thái rõ ràng: `Đang lưu...`, `Đã lưu lúc HH:mm`,
  `Lỗi lưu - sẽ thử lại`. Định dạng giờ theo `DESIGN_SYSTEM.md` mục 4.
- `SHOULD` lưu tạm cục bộ (localStorage hoặc IndexedDB) làm lớp dự phòng khi mất
  mạng, đồng bộ lại khi có mạng.
- `SHOULD` giữ lịch sử phiên bản để khôi phục bản trước.

### 5.2. Dán thông minh

- `MUST` tự động làm sạch khi dán từ Microsoft Word, Google Docs, hoặc trang web
  khác: bỏ style và class thừa, bỏ `<span>` lồng nhau vô nghĩa, bỏ comment ẩn
  của Word (`<!--[if ...]-->`), bỏ thuộc tính `style` nội tuyến không cần thiết.
- `MUST` **giữ lại định dạng có ý nghĩa** khi làm sạch: đậm, nghiêng, tiêu đề,
  danh sách, liên kết, bảng. Xóa trắng toàn bộ định dạng là hỏng chức năng, không
  phải là làm sạch.
- `SHOULD` có tùy chọn "Dán dạng văn bản thuần" qua `Ctrl/Cmd+Shift+V`.

---

## 6. Xử lý hình ảnh và tệp

- `MUST` tự động nén ảnh tải lên và chuyển sang **WebP** (giữ bản dự phòng
  JPEG/PNG cho trình duyệt không hỗ trợ nếu cần).
- `MUST` sinh nhiều kích thước cho `srcset`, tối thiểu: thu nhỏ, trung bình, đầy
  đủ. `MUST NOT` gửi ảnh gốc kích thước lớn cho mọi thiết bị.
- `MUST` lưu ảnh trên máy chủ tự vận hành **hoặc** dịch vụ lưu trữ trong danh
  sách 0 đồng (ưu tiên **Cloudflare R2**). Cấu hình `MUST` khai qua biến môi
  trường, `MUST NOT` hard-code đường dẫn.
- `MUST` giới hạn dung lượng tệp tải lên (mặc định đề xuất 5MB mỗi ảnh) và kiểm
  tra định dạng bằng **magic number**, không dựa vào phần mở rộng hay
  `Content-Type` do trình duyệt gửi.
- `MUST` sinh lại tên tệp ở phía máy chủ. `MUST NOT` dùng tên tệp do người dùng
  đặt để ghi lên đĩa - đây là đường dẫn tới lỗi vượt thư mục (path traversal).
- `MUST` phục vụ tệp tải lên từ một tên miền hoặc đường dẫn **không có cookie
  phiên**, để một tệp HTML tải lên không chạy được trong ngữ cảnh đăng nhập.

---

## 7. Bảo mật

> Trình soạn thảo là bề mặt tấn công XSS lớn nhất của một hệ thống nội dung.
> Toàn bộ mục này là `MUST`, không có ngoại lệ.

- `MUST` **làm sạch XSS ở tầng backend** trước khi lưu vào cơ sở dữ liệu. `MUST
  NOT` tin vào việc làm sạch ở frontend - kẻ tấn công gọi thẳng API, không đi qua
  giao diện của bạn.
  Thư viện gợi ý: `DOMPurify` (chạy phía máy chủ qua `jsdom`), `sanitize-html`,
  hoặc bộ lọc HTML theo danh sách trắng tương đương của ngôn ngữ backend.
- `MUST` dùng **danh sách trắng** thẻ và thuộc tính, `MUST NOT` dùng danh sách
  đen. Chỉ cho phép các thẻ tương ứng với tính năng ở mục 3 và 4.
- `MUST` loại bỏ hoàn toàn: thẻ `<script>`, `<iframe>` không thuộc danh sách
  nhúng được duyệt, mọi thuộc tính `on*` (`onclick`, `onerror`, `onload`...),
  giao thức `javascript:` và `data:text/html` trong liên kết và ảnh.
- `MUST` **làm sạch lại một lần nữa ở tầng hiển thị** trước khi đưa vào DOM của
  trang công khai. Đây là lớp phòng thủ thứ hai theo `SECURITY_BASELINE.md` mục 1.2:
  dữ liệu có thể đã nằm trong database từ trước khi bộ lọc được siết.
- `MUST` kiểm tra quyền trước khi cho truy cập tính năng **Xem và sửa mã nguồn
  HTML** (mục 3.5). Tính năng này cho phép người dùng viết HTML tùy ý, nên nó
  tương đương quyền quản trị nội dung.
- `MUST` kiểm tra và giới hạn loại tệp được tải lên, quét mã độc nếu hệ thống có
  sẵn công cụ quét.
- `MUST` đặt `Content-Security-Policy` theo `SECURITY_BASELINE.md` mục 7.2. CSP
  chặt là lớp phòng thủ cuối cùng khi bộ lọc HTML sót một trường hợp.
- `MUST` giới hạn tần suất cho endpoint lưu nội dung và tải tệp lên.

---

## 8. Kiến trúc dữ liệu và đầu ra

### 8.1. Lược đồ tối thiểu cho bảng nội dung

| Trường | Kiểu | Mô tả | Mức |
| --- | --- | --- | --- |
| `id` | UUID / BigInt | Khóa chính | `M` |
| `title` | string | Tiêu đề gốc, có dấu | `M` |
| `content_html` | text / JSON | Nội dung định dạng gốc, dùng để hiển thị | `M` |
| `plain_text_content` | text | Đã bỏ thẻ HTML, dùng cho đoạn xem trước | `M` |
| `search_normalized_content` | text | Đã chuẩn hóa theo `SEARCH_AND_FILTER.md` mục 3, dùng để lập chỉ mục | `M` |
| `search_normalized_title` | text | Tiêu đề đã chuẩn hóa tương tự | `M` |
| `slug` | string | Đường dẫn thân thiện, không dấu, không khoảng trắng | `M` |
| `category_id` / `tag_ids` | tham chiếu | Phục vụ lọc | `M` |
| `status` | enum | `draft` / `published` / `archived` | `M` |
| `word_count` | int | Số từ theo quy tắc tách từ tiếng Việt | `S` |
| `created_at`, `updated_at`, `published_at` | timestamp | | `M` |

### 8.2. Vòng đời cập nhật chỉ mục

- `MUST` mỗi khi `content_html` hoặc `title` thay đổi, các trường phái sinh
  (`plain_text_content`, `search_normalized_content`, `search_normalized_title`,
  `word_count`) `MUST` được tính lại **trong cùng một giao dịch**, hoặc qua tác
  vụ bất đồng bộ **có cơ chế thử lại** nếu dùng công cụ tìm kiếm ngoài.
- `MUST` nếu dùng công cụ tìm kiếm ngoài (Elasticsearch, Meilisearch, Typesense),
  `MUST` có tác vụ lập chỉ mục lại toàn bộ, chạy được thủ công và theo lịch, để
  xử lý khi chỉ mục lệch với cơ sở dữ liệu.

### 8.3. Đầu ra

- `MUST` trình soạn thảo trả nội dung ở dạng **HTML chuẩn** và/hoặc **JSON có
  cấu trúc** (Tiptap/ProseMirror mặc định dùng JSON, xuất được HTML).
- `MUST` trang công khai hiển thị **đúng y hệt** bản đã soạn: không lệch phông,
  không lệch khoảng cách, không mất bảng biểu. Đây là ý nghĩa thật của WYSIWYG.
- `SHOULD` lưu JSON làm nguồn chân lý và sinh HTML khi hiển thị, thay vì lưu HTML
  làm nguồn - JSON có cấu trúc dễ di chuyển và dễ kiểm tra hơn.

---

## 9. Cấu trúc thư mục và đặt tên

`SHOULD` áp dụng cấu trúc sau cho mọi project tích hợp editor và tìm kiếm, điều
chỉnh theo framework nhưng giữ nguyên cách nhóm chức năng:

```
src/
  modules/
    content-editor/
      RichTextEditor.{tsx,vue}       # Component editor chính
      extensions/                    # Phần mở rộng riêng (ảnh, video, bảng...)
      toolbar/                       # Cấu hình và component thanh công cụ
      sanitize/                      # Làm sạch phía client (lớp bổ sung, KHÔNG thay backend)
      utils/
        vi-word-count.ts             # Đếm từ tiếng Việt, dùng chung
    content-search/
      normalize/
        vi-text-normalizer.ts        # Chuẩn hóa tiếng Việt (bỏ dấu, NFC...)
      api/
        search.client.ts             # Client gọi API tìm/lọc
      filters/                       # Component bộ lọc dùng chung
```

- `MUST` đặt tên hàm và biến xử lý tiếng Việt theo tiền tố `vi` hoặc
  `Vietnamese`: `viNormalizeText()`, `viRemoveDiacritics()`, `viWordCount()`.
  Nhờ vậy không lẫn với logic xử lý văn bản chung.
- `MUST NOT` viết lặp logic chuẩn hóa tiếng Việt ở nhiều nơi. `MUST` gom vào một
  module dùng chung (`content-search/normalize`) và nhập lại từ đó.

---

## 10. Khả năng truy cập và responsive

- `MUST` thanh công cụ **rút gọn thông minh** trên màn hình nhỏ: gom nhóm nút vào
  menu "Thêm", ưu tiên hiển thị các nút dùng nhiều nhất.
- `MUST` mọi nút chức năng có `aria-label` bằng tiếng Việt và đi được bằng
  `Tab` / `Shift+Tab`.
- `MUST` ảnh chèn vào bài bắt buộc có văn bản thay thế trước khi cho phép xuất
  bản - kiểm tra ở bước xuất bản, không chỉ nhắc nhở.
- `MUST` vùng soạn thảo có nhãn gắn đúng và thông báo được cho trình đọc màn hình
  khi đổi định dạng.
- `MUST` tương phản của thanh công cụ đạt ngưỡng tại
  [`ACCESSIBILITY.md`](ACCESSIBILITY.md) mục 2, ở cả ba chế độ Sáng / Ấm / Tối.

---

## 11. Tiêu chí nghiệm thu

Một tính năng liên quan tới editor chỉ được coi là xong khi:

- [ ] Toàn bộ tính năng mức `M` ở mục 3 và 4 hoạt động đúng trên desktop và di động.
- [ ] Gõ tiếng Việt có dấu mượt mà, không mất ký tự, không nhảy con trỏ - đã thử
      với ít nhất hai bộ gõ khác nhau.
- [ ] Dán từ Word và Google Docs được làm sạch đúng mục 5.2, kiểm tra HTML sinh
      ra không còn style rác.
- [ ] Ảnh tải lên tự nén sang WebP và có nhiều kích thước responsive.
- [ ] Nội dung lưu vào cơ sở dữ liệu đã qua làm sạch XSS ở backend - **có ca kiểm
      thử** chèn `<script>`, `onerror`, `javascript:` và xác nhận bị loại bỏ.
- [ ] Trang công khai hiển thị đúng 100% so với bản soạn thảo.
- [ ] Tự động lưu chạy đúng chu kỳ và báo trạng thái rõ ràng.
- [ ] Tính năng xem mã nguồn HTML chỉ hiện với vai trò có quyền.
- [ ] Đã rà soát bảo mật cho luồng tải tệp lên và luồng lưu nội dung HTML theo
      [`templates/SECURITY_REVIEW.md`](templates/SECURITY_REVIEW.md).
- [ ] Đạt tiêu chí nghiệm thu của [`ACCESSIBILITY.md`](ACCESSIBILITY.md) mục 8.

---

## 12. Yêu cầu kiểm thử

| Loại | Mức | Nội dung |
| --- | --- | --- |
| Unit | `M` | Hàm chuẩn hóa tiếng Việt, hàm đếm từ, hàm làm sạch HTML |
| Integration | `M` | Luồng tải ảnh lên -> nén -> lưu; luồng lưu nội dung -> làm sạch -> đọc lại |
| Bảo mật | `M` | Chèn các mẫu XSS phổ biến, xác nhận bị loại bỏ **cả khi lưu và khi hiển thị** |
| E2E | `S` | Soạn bài đủ định dạng -> xuất bản -> tìm kiếm không dấu -> xác nhận hiển thị đúng |
| Thủ công đa trình duyệt | `M` | Chrome, Firefox, Safari, Edge trên desktop và di động, gõ bằng bộ gõ thật |
| Khả năng truy cập | `M` | `axe-core` trên màn hình soạn thảo, đi hết thanh công cụ bằng bàn phím |

---

## 13. Tài liệu tham khảo

- Tiptap: https://tiptap.dev/
- ProseMirror: https://prosemirror.net/
- CKEditor 5 (mã nguồn mở): https://ckeditor.com/ckeditor-5/
- TinyMCE: https://www.tiny.cloud/
- Quill.js: https://quilljs.com/
- Lexical: https://lexical.dev/
- DOMPurify: https://github.com/cure53/DOMPurify
- sanitize-html: https://github.com/apostrophecms/sanitize-html
- OWASP XSS Prevention Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html

---

> **Lịch sử**
>
> | Phiên bản | Ngày | Nội dung |
> | --- | --- | --- |
> | 2.0.0 | 24/08/2026 | Đưa vào bộ quy ước trung tâm. Tách phần tìm/lọc sang `SEARCH_AND_FILTER.md`, bổ sung mục bảng biểu, siết yêu cầu bảo mật tệp tải lên, thêm Lexical và Tauri vào danh sách được duyệt |
> | 1.0 | 23/08/2026 | Bản RFC đầu tiên, tổng hợp yêu cầu editor và hệ thống tìm/lọc tiếng Việt |
