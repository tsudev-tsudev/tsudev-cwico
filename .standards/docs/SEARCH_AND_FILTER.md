# CHUẨN TÌM KIẾM VÀ LỌC TỐI ƯU TIẾNG VIỆT - v3.0.0

> **Bắt buộc** với mọi chức năng tìm kiếm hoặc lọc nội dung trong hệ sinh thái
> tsudev, ở cả trang quản trị lẫn trang người dùng.
>
> Tài liệu này **gộp và thay thế** hai tài liệu rời trước đây: yêu cầu chức năng
> tìm kiếm/lọc và mục "Hệ thống tìm/lọc tối ưu tiếng Việt" của RFC editor.
>
> Ký hiệu: `MUST` / `MUST NOT` = bắt buộc. `SHOULD` = nên, bỏ qua phải ghi lý do.
> `MAY` = tùy chọn.

## Mục lục

- [1. Mục tiêu và nguyên tắc kiến trúc](#1-mục-tiêu-và-nguyên-tắc-kiến-trúc)
- [2. Giao diện và trải nghiệm](#2-giao-diện-và-trải-nghiệm)
- [3. Chuẩn hóa dữ liệu tiếng Việt](#3-chuẩn-hóa-dữ-liệu-tiếng-việt)
- [4. Tìm kiếm không dấu](#4-tìm-kiếm-không-dấu)
- [5. Xếp hạng kết quả](#5-xếp-hạng-kết-quả)
- [6. Bộ lọc](#6-bộ-lọc)
- [7. Chuẩn API tìm và lọc](#7-chuẩn-api-tìm-và-lọc)
- [8. Chọn công nghệ theo quy mô](#8-chọn-công-nghệ-theo-quy-mô)
- [9. Lập chỉ mục](#9-lập-chỉ-mục)
- [10. Yêu cầu phi chức năng](#10-yêu-cầu-phi-chức-năng)
- [11. Tiêu chí nghiệm thu](#11-tiêu-chí-nghiệm-thu)
- [12. Yêu cầu kiểm thử](#12-yêu-cầu-kiểm-thử)
- [13. Lộ trình triển khai](#13-lộ-trình-triển-khai)

---

## 1. Mục tiêu và nguyên tắc kiến trúc

**Mục tiêu.** Tìm kiếm và lọc theo thời gian thực, xử lý tốt tiếng Việt có dấu
lẫn không dấu, trả kết quả chính xác - nhanh - phân nhóm rõ ràng.

**Bốn nguyên tắc bắt buộc:**

1. `MUST` tách riêng **tầng lưu trữ nội dung gốc** (HTML/JSON, dùng để hiển thị)
   và **tầng chỉ mục tìm kiếm** (đã chuẩn hóa, dùng để tìm và lọc).
   `MUST NOT` tìm kiếm trực tiếp trên cột HTML thô.
2. `MUST NOT` triển khai tìm kiếm kiểu `LIKE '%từ khóa%'` đơn thuần. Cách đó
   không tìm được không dấu, không xếp hạng được, và quét toàn bảng.
3. `MUST` khi lưu hoặc cập nhật nội dung, hệ thống tự sinh `plain_text_content`
   (đã bỏ thẻ HTML) và `search_normalized_content` (đã chuẩn hóa theo mục 3).
4. `MUST` API tìm kiếm là **không trạng thái**, nhận tham số theo đúng chuẩn ở
   mục 7 - không để mỗi module tự đặt tên tham số riêng.

**Phạm vi dữ liệu được lập chỉ mục** (điều chỉnh theo từng hệ thống):

| Loại nội dung | Trường được lập chỉ mục |
| --- | --- |
| Bài viết | Tiêu đề, tóm tắt, nội dung, thẻ, danh mục |
| Công cụ / tính năng | Tên, mô tả ngắn, mô tả chi tiết, đường dẫn, từ khóa liên quan |
| Danh mục | Tên danh mục, mô tả |

---

## 2. Giao diện và trải nghiệm

### 2.1. Tìm kiếm tức thời

- `MUST` bắt đầu hiển thị kết quả ngay khi người dùng gõ, kích hoạt truy vấn từ
  **2 ký tự** trở lên (cấu hình được qua `MIN_QUERY_LENGTH`, mặc định 2).
- `MUST NOT` gửi yêu cầu khi ô tìm kiếm trống hoặc chỉ có khoảng trắng.

### 2.2. Kỹ thuật giảm dồn yêu cầu (debounce)

- `MUST` áp dụng debounce **300 đến 500ms** sau khi người dùng ngừng gõ, mặc
  định đề xuất 350ms.
- `MUST` hủy yêu cầu cũ đang chờ khi người dùng gõ tiếp (`AbortController` ở
  frontend, hoặc hủy truy vấn ở backend). Không làm việc này sẽ sinh **tranh
  chấp thứ tự**: kết quả của truy vấn cũ về sau và đè lên kết quả mới.

### 2.3. Khung kết quả

- `MUST` **phân nhóm theo loại nội dung**, mỗi nhóm hiển thị tối đa N kết quả
  (đề xuất 5) kèm liên kết "Xem tất cả kết quả":
  - Bài viết
  - Công cụ / tính năng
  - Danh mục
- `MUST` **tô sáng từ khóa** khớp trong tiêu đề và mô tả, dùng thẻ `<mark>` hoặc
  class riêng. `MUST` tô sáng đúng cả khi người dùng gõ **không dấu** mà văn bản
  gốc **có dấu** - đây là chỗ hầu hết cách triển khai làm sai, vì vị trí ký tự
  của chuỗi đã chuẩn hóa không trùng với chuỗi gốc. `MUST` ánh xạ ngược vị trí về
  chuỗi gốc, không tô sáng trên chuỗi đã chuẩn hóa.
- `MUST` có trạng thái **đang tải**: khung xương (skeleton) kích thước cố định,
  không làm nhảy bố cục.
- `MUST` có trạng thái **rỗng**: `Không tìm thấy kết quả phù hợp với "{từ khóa}"`
  kèm gợi ý cụ thể (kiểm tra chính tả, thử từ khóa khác, hoặc gợi ý từ khóa phổ biến).
- `MUST` có trạng thái **lỗi** thân thiện khi API gặp sự cố. `MUST NOT` để trắng
  hoặc treo giao diện.

### 2.4. Điều hướng bằng bàn phím

| Phím | Hành vi |
| --- | --- |
| `↓` | Chuyển xuống kết quả tiếp theo |
| `↑` | Chuyển lên kết quả trước |
| `Enter` | Mở kết quả đang chọn; nếu chưa chọn mục nào thì mở trang kết quả đầy đủ |
| `Esc` | Đóng khung kết quả, **giữ nguyên** nội dung đã gõ |
| `Ctrl/Cmd + K` | Mở nhanh ô tìm kiếm từ bất kỳ đâu trên trang (`SHOULD`) |

- `MUST` mục đang chọn có trạng thái nhìn thấy rõ và **cuộn vào vùng nhìn thấy**
  khi danh sách dài.
- `MUST` dùng đúng bộ ARIA: `role="combobox"`, `aria-expanded`, `aria-controls`,
  `aria-activedescendant`, danh sách `role="listbox"`, mục `role="option"`. Chi
  tiết tại [`ACCESSIBILITY.md`](ACCESSIBILITY.md) mục 4.

### 2.5. Trải nghiệm bổ sung

- `SHOULD` lưu **lịch sử tìm kiếm gần đây** ở trình duyệt (localStorage), hiển
  thị khi ô tìm kiếm nhận focus mà chưa gõ gì. `MUST` có cách xóa lịch sử này.
- `MAY` gợi ý từ khóa phổ biến hoặc đang được quan tâm.
- `MUST` responsive: trên màn hình nhỏ, khung kết quả chuyển thành lớp phủ toàn
  màn hình thay vì menu thả xuống.

---

## 3. Chuẩn hóa dữ liệu tiếng Việt

### 3.1. Pipeline chuẩn hóa

Khi sinh `search_normalized_content` và `search_normalized_title`, `MUST` thực
hiện **đúng thứ tự** sau:

1. **Bỏ thẻ HTML** - giữ lại văn bản thuần và `alt` của ảnh.
2. **Chuẩn hóa Unicode về NFC** - bắt buộc. Tiếng Việt có hai cách tổ hợp dấu
   (dựng sẵn và tổ hợp); bỏ bước này thì hai chuỗi trông y hệt nhau lại không
   khớp nhau.
3. **Chuyển về chữ thường**.
4. **Sinh thêm bản không dấu** song song với bản có dấu: `"giáo dục"` sinh thêm
   `"giao duc"`.
5. **Chuẩn hóa khoảng trắng** - gộp khoảng trắng thừa, bỏ ký tự xuống dòng dư.
6. **Bỏ dấu câu** không cần cho việc so khớp. Bản gốc vẫn giữ nguyên dấu câu để
   hiển thị.

### 3.2. Cài đặt bỏ dấu

Chữ `đ` và `Đ` **không tách được qua NFD** - đây là cái bẫy phổ biến nhất khi
xử lý tiếng Việt. `MUST` xử lý riêng:

```ts
/** Bỏ dấu tiếng Việt. Xử lý riêng đ/Đ vì NFD không tách được hai ký tự này. */
export function viRemoveDiacritics(input: string): string {
  return input
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/đ/g, 'd')
    .replace(/Đ/g, 'D');
}

/** Chuẩn hóa chuỗi để so khớp: NFC -> chữ thường -> bỏ dấu -> gộp khoảng trắng. */
export function viNormalizeText(input: string): string {
  return viRemoveDiacritics(input.normalize('NFC').toLowerCase())
    .replace(/\s+/g, ' ')
    .trim();
}
```

`MUST` đặt hai hàm này trong **một** module dùng chung và nhập lại từ đó. `MUST
NOT` viết lặp ở nhiều nơi - mỗi bản sao là một cơ hội để hai chỗ chuẩn hóa khác
nhau và kết quả tìm kiếm mâu thuẫn.

### 3.3. Tách từ tiếng Việt

Tiếng Việt là ngôn ngữ đơn lập: từ ghép gồm nhiều âm tiết cách nhau bởi khoảng
trắng ("học sinh" là một từ, hai âm tiết). Vì vậy:

- `MUST` dùng bộ tách từ tiếng Việt khi lập chỉ mục, để tìm đúng theo **từ** chứ
  không chỉ theo âm tiết rời. Công cụ gợi ý: `underthesea`, `pyvi`, `vntk`
  (Python), hoặc bộ phân tích tiếng Việt cho Elasticsearch (ICU Analyzer kèm từ
  điển riêng, hoặc `elasticsearch-analysis-vietnamese`).
- `MUST` chức năng đếm từ theo thời gian thực của trình soạn thảo
  ([`RICH_TEXT_EDITOR.md`](RICH_TEXT_EDITOR.md) mục 3.5) dùng **cùng một logic**.
  Đếm theo cụm cách nhau bởi khoảng trắng là mức tối thiểu chấp nhận được ở tầng
  giao diện; tách từ theo ngữ nghĩa là mức nâng cao, `SHOULD` có khi đã dùng
  chung engine với tìm kiếm.

---

## 4. Tìm kiếm không dấu

Đây là yêu cầu quan trọng nhất về mặt logic, vì phần lớn người dùng Việt Nam gõ
không dấu khi tìm kiếm.

- `MUST` gõ **không dấu** vẫn ra kết quả **có dấu**: gõ `"giao duc"` phải trả về
  bài viết chứa `"giáo dục"`.
- `MUST` chiều ngược lại vẫn đúng, và **kết quả khớp chính xác cả dấu `MUST`
  được xếp hạng cao hơn** kết quả chỉ khớp phần không dấu, khi mức độ khớp tương
  đương.
- `MUST` không phân biệt hoa thường.
- `MUST` xử lý đúng các trường hợp đặc thù chính tả tiếng Việt: `d` và `đ`, biến
  thể gõ telex phổ biến (gõ `"thu vien"` vẫn ra `"thư viện"`).
- `SHOULD` hỗ trợ **tìm gần đúng** (chấp nhận gõ sai 1 đến 2 ký tự) cho từ dài từ
  4 ký tự trở lên. Cài đặt bằng khoảng cách chuỗi (Levenshtein) hoặc bằng khả
  năng chịu lỗi gõ có sẵn của Meilisearch, Typesense, Elasticsearch.

**Cách lưu:** `MUST` lưu song song hai trường trong chỉ mục - trường gốc (có dấu,
dùng để hiển thị và tô sáng) và trường `*_normalized` (không dấu, dùng để so
khớp). `MUST` tính sẵn trường chuẩn hóa **tại thời điểm ghi dữ liệu**, không
tính lại mỗi lần tìm kiếm.

---

## 5. Xếp hạng kết quả

Áp dụng thang điểm giảm dần:

| Hạng | Tiêu chí khớp | Trọng số đề xuất |
| --- | --- | --- |
| 1 | Khớp chính xác tuyệt đối tên công cụ hoặc tiêu đề bài viết | 100 |
| 2 | Từ khóa nằm ở đầu tiêu đề | 70 |
| 3 | Từ khóa xuất hiện trong tiêu đề | 50 |
| 4 | Khớp trong thẻ hoặc tóm tắt | 30 |
| 5 | Khớp trong nội dung chi tiết | 10 |
| - | Khớp gần đúng | Nhân hệ số giảm, đề xuất 0.5, so với khớp chính xác cùng hạng |
| - | Chỉ khớp bản không dấu (bản có dấu không khớp) | Nhân hệ số giảm, đề xuất 0.8 |

- Kết quả `MUST` sắp xếp giảm dần theo tổng điểm.
- Khi điểm bằng nhau, `MUST` có tiêu chí phụ xác định: nội dung mới hơn trước,
  hoặc lượt xem cao hơn trước. Không có tiêu chí phụ thì thứ tự sẽ đổi giữa các
  lần tải trang và người dùng mất niềm tin vào kết quả.

---

## 6. Bộ lọc

### 6.1. Các trục lọc

`MUST` hỗ trợ tối thiểu các trục sau, **kết hợp được đồng thời** (AND giữa các
nhóm, OR trong cùng một nhóm):

| Nhóm lọc | Chi tiết | Mức |
| --- | --- | --- |
| Loại nội dung | Bài viết / Công cụ / Danh mục, chọn nhiều | `M` |
| Danh mục | Chọn nhiều theo phân loại hiện có | `M` |
| Thẻ | Chọn nhiều | `M` |
| Tác giả | | `M` |
| Trạng thái | Nháp / Đã xuất bản / Lưu trữ. Trạng thái nội bộ **chỉ hiển thị cho vai trò có quyền** | `M` |
| Khoảng thời gian | Ngày tạo / cập nhật / xuất bản. Có sẵn 7 ngày qua, 1 tháng qua, 1 năm qua, và khoảng tùy chọn | `M` |
| Ngôn ngữ nội dung | Khi hệ thống đa ngôn ngữ | `S` |

### 6.2. Sắp xếp

`MUST` cho người dùng chọn: **Độ liên quan** (mặc định), **Mới nhất**, **Cũ
nhất**, **Xem nhiều nhất**.

### 6.3. Quy tắc bắt buộc

- `MUST` mọi bộ lọc hoạt động **cùng một truy vấn** với ô tìm kiếm từ khóa.
  `MUST NOT` tách thành hai luồng xử lý riêng.
- `MUST` hiển thị **số lượng kết quả dự kiến** cho từng lựa chọn khi khả thi
  (facet count), để người dùng biết trước lọc có ra kết quả hay không.
- `MUST` có trạng thái "không có kết quả" rõ ràng kèm gợi ý nới lỏng bộ lọc.
- `MUST` **kiểm tra quyền ở phía máy chủ** cho các giá trị lọc nhạy cảm. Người
  dùng thường tự thêm `&status=draft` vào URL - đây là ca IDOR kinh điển theo
  [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 6.3.

### 6.4. Trang kết quả đầy đủ

- `MUST` khi người dùng nhấn `Enter` hoặc "Xem tất cả kết quả", điều hướng tới
  `/search?q=...&type=...&category=...&sort=...` hiển thị đầy đủ, có phân trang
  hoặc cuộn vô hạn.
- `MUST` phản ánh từ khóa và bộ lọc qua **tham số URL**, để chia sẻ được liên kết
  và nút quay lại/tiến tới của trình duyệt trả về đúng trạng thái.
- `MUST` hiển thị số lượng kết quả theo từng loại, ví dụ "12 bài viết, 3 công cụ".

---

## 7. Chuẩn API tìm và lọc

`MUST` mọi endpoint tìm kiếm tuân theo đúng bộ tham số sau, áp dụng toàn hệ thống:

```
GET /api/v1/{resource}?q={từ khóa}&type={slug}&category={slug}&tag={slug1,slug2}
    &author={id}&status={draft|published|archived}
    &date_from={ISO8601}&date_to={ISO8601}
    &sort={relevance|newest|oldest|popular}&page={n}&page_size={n}
```

Phản hồi `MUST` kèm thông tin phân trang và (khi áp dụng) số đếm theo nhóm lọc:

```json
{
  "data": [],
  "meta": {
    "total": 128,
    "page": 1,
    "page_size": 10,
    "query_normalized": "giao duc"
  },
  "facets": {
    "category": [{ "slug": "tin-tuc", "count": 42 }],
    "tag": [{ "slug": "hoc-bong", "count": 15 }]
  }
}
```

- `MUST` `page_size` chỉ nhận các mốc chuẩn `{10, 20, 50, 100, 200}` với mặc
  định `10`, và có **trần cứng ở phía máy chủ là `200`**. Bộ mốc, vị trí bộ chọn
  trên giao diện và hành vi khi đổi mốc quy định ở
  [`DATA_TABLE.md`](DATA_TABLE.md). Không đặt trần là mở cửa cho tấn công cạn
  tài nguyên bằng `page_size=1000000`.
- Trần `200` đi kèm hai ràng buộc bù, `MUST` có đủ cả hai:
  1. `page_size` từ `100` trở lên có **giới hạn tần suất riêng, chặt hơn** (đề
     xuất 10 yêu cầu / phút / tài khoản).
  2. Tập dữ liệu vượt **100.000 bản ghi** `MUST` chuyển sang **phân trang con
     trỏ**, xem [`DATA_TABLE.md`](DATA_TABLE.md) mục 8.3. `OFFSET` lớn buộc cơ
     sở dữ liệu quét rồi bỏ đi từng dòng một.
- Cần nhiều hơn 200 bản ghi cùng lúc thì `MUST` dùng chức năng xuất tệp chạy
  nền, `MUST NOT` nới trần API.
- `MUST` bỏ qua, chứ không báo lỗi, với tham số lọc không hợp lệ - trừ khi tham
  số đó ảnh hưởng tới quyền, khi đó `MUST` từ chối.
- `MUST` gắn phiên bản vào đường dẫn theo [`VERSIONING.md`](VERSIONING.md) mục 4.

---

## 8. Chọn công nghệ theo quy mô

| Quy mô dữ liệu | Đề xuất | Chi phí |
| --- | --- | --- |
| Nhỏ (dưới vài nghìn bản ghi) | Tìm trực tiếp trên cơ sở dữ liệu với cột `*_normalized` đã lập chỉ mục; hoặc **Fuse.js** phía client | 0 đồng |
| Nhỏ, dùng PostgreSQL | **PostgreSQL Full-Text Search** kèm extension `unaccent` và `pg_trgm` | 0 đồng |
| Trung bình đến lớn | **Meilisearch** hoặc **Typesense** - có sẵn khả năng chịu lỗi gõ, tốc độ tức thời, tích hợp tiếng Việt dễ | 0 đồng khi tự vận hành trên Oracle Cloud Always Free |
| Rất lớn | **Elasticsearch/OpenSearch** với bộ phân tích tiếng Việt riêng | 0 đồng khi tự vận hành, nhưng tốn tài nguyên máy |

`MUST` đối chiếu lựa chọn với [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md) trước
khi quyết định. Với phần lớn project tsudev, **PostgreSQL Full-Text Search hoặc
Meilisearch là đủ** - đừng dựng Elasticsearch cho vài nghìn bài viết.

---

## 9. Lập chỉ mục

- `MUST` đồng bộ lại dữ liệu tìm kiếm ngay khi nội dung được tạo, cập nhật, hoặc
  xóa - theo sự kiện, hoặc theo tác vụ định kỳ.
- `MUST` tính sẵn trường `*_normalized` **tại thời điểm ghi dữ liệu**, không tính
  lại mỗi lần tìm kiếm.
- `MUST` có tác vụ **lập chỉ mục lại toàn bộ**, chạy được thủ công, để xử lý khi
  chỉ mục lệch với cơ sở dữ liệu. Chỉ mục sẽ lệch - vấn đề chỉ là khi nào.
- `MUST` khi dùng công cụ tìm kiếm ngoài, việc cập nhật chỉ mục `MUST` có cơ chế
  thử lại. Ghi vào database thành công mà cập nhật chỉ mục thất bại lặng lẽ là
  lỗi khó phát hiện nhất trong nhóm này.

---

## 10. Yêu cầu phi chức năng

### 10.1. Hiệu năng

- `MUST` thời gian phản hồi trung bình của API tìm kiếm **dưới 300ms** với tập dữ
  liệu tới 100.000 bản ghi, đo ở môi trường staging.
- `MUST` có chỉ mục cơ sở dữ liệu cho mọi cột dùng để lọc và sắp xếp.
- `SHOULD` cache kết quả của các truy vấn phổ biến với thời gian sống ngắn
  (1 đến 5 phút).

### 10.2. Bảo mật

- `MUST` giới hạn tần suất cho endpoint tìm kiếm, kết hợp với debounce ở
  frontend. Tìm kiếm là endpoint tốn tài nguyên nhất và dễ bị lạm dụng nhất.
- `MUST` dùng **truy vấn tham số hóa**. Ghép chuỗi từ khóa vào câu SQL là lỗ hổng
  tiêm mã SQL trực tiếp.
- `MUST` mã hóa đầu ra khi tô sáng từ khóa vào HTML. Từ khóa là dữ liệu do người
  dùng nhập; chèn thẳng vào `innerHTML` là lỗ hổng XSS phản xạ.
- `MUST` lọc kết quả theo quyền của người gọi **trước khi** trả về, không lọc ở
  phía giao diện.

### 10.3. Theo dõi và cải thiện

- `SHOULD` ghi lại các từ khóa được tìm, đặc biệt là từ khóa **không ra kết quả**
  - đây là danh sách nội dung cần bổ sung, có giá trị trực tiếp cho SEO.
- `MUST NOT` ghi kèm định danh người dùng vào nhật ký từ khóa nếu chưa có cơ sở
  hợp pháp; đây là dữ liệu mức D2 theo `SECURITY_BASELINE.md` mục 2.

### 10.4. Khả năng mở rộng

Thiết kế `MUST` cho phép thêm loại nội dung mới vào phạm vi tìm kiếm (người dùng,
bình luận, tài liệu...) mà không phải đổi kiến trúc.

---

## 11. Tiêu chí nghiệm thu

- [ ] Gõ từ 2 ký tự trở lên thì kết quả tự cập nhật, có debounce, không giật.
- [ ] Gõ không dấu (`"dinh dang"`) trả đúng kết quả có dấu (`"Định dạng"`).
- [ ] Gõ sai chính tả nhẹ (`"dinh dnag"`) vẫn ra gợi ý gần đúng.
- [ ] Kết quả khớp chính xác cả dấu được xếp trên kết quả chỉ khớp không dấu.
- [ ] Kết quả phân nhóm rõ theo loại nội dung.
- [ ] Từ khóa được tô sáng đúng vị trí, **kể cả khi gõ không dấu**.
- [ ] Có đủ trạng thái: đang tải, không có kết quả, lỗi.
- [ ] Điều hướng bằng bàn phím (`↑ ↓ Enter Esc`) hoạt động đầy đủ, focus rõ ràng.
- [ ] Kết hợp được từ khóa với bộ lọc loại/danh mục/thẻ/tác giả/ngày/trạng thái.
- [ ] Sắp xếp được theo Liên quan / Mới nhất / Cũ nhất / Xem nhiều nhất.
- [ ] Trang kết quả đầy đủ phản ánh trạng thái qua URL và chia sẻ được.
- [ ] Người dùng không có quyền `MUST NOT` thấy được nội dung nháp qua tham số URL.
- [ ] API phản hồi dưới 300ms ở staging với tập dữ liệu mục tiêu.
- [ ] Đạt tiêu chí nghiệm thu của [`ACCESSIBILITY.md`](ACCESSIBILITY.md) mục 8.

---

## 12. Yêu cầu kiểm thử

| Loại | Mức | Nội dung |
| --- | --- | --- |
| Unit | `M` | `viNormalizeText`, `viRemoveDiacritics`, `viWordCount`, hàm tính điểm xếp hạng |
| Unit | `M` | Ca đặc thù: `đ/Đ`, chuỗi NFC so với NFD, chữ hoa có dấu, chuỗi rỗng, chuỗi chỉ có dấu câu |
| Integration | `M` | API tìm/lọc với các tổ hợp bộ lọc khác nhau, kể cả tổ hợp không ra kết quả |
| Bảo mật | `M` | Tiêm mã SQL qua tham số `q`; XSS qua phần tô sáng; vượt quyền qua `status=draft` |
| Hiệu năng | `S` | Đo thời gian phản hồi với tập dữ liệu bằng quy mô mục tiêu |
| E2E | `S` | Gõ không dấu -> chọn bằng bàn phím -> mở kết quả -> quay lại giữ nguyên trạng thái |

---

## 13. Lộ trình triển khai

Ba giai đoạn, làm theo thứ tự:

1. **Giai đoạn 1 - nền tảng.** Chuẩn hóa dữ liệu tiếng Việt (thêm trường
   `*_normalized`), API tìm kiếm cơ bản có debounce, tô sáng, phân nhóm kết quả.
2. **Giai đoạn 2 - chất lượng kết quả.** Tìm gần đúng, xếp hạng theo điểm, điều
   hướng bàn phím đầy đủ, khả năng truy cập.
3. **Giai đoạn 3 - chiều sâu.** Bộ lọc nhiều trục, số đếm theo nhóm, sắp xếp,
   trang kết quả đầy đủ, lịch sử tìm kiếm, thống kê từ khóa.

`MUST NOT` nhảy sang giai đoạn 3 khi giai đoạn 1 chưa xong. Bộ lọc đẹp trên nền
chuẩn hóa sai chỉ cho ra kết quả sai nhanh hơn.

---

> **Lịch sử**
>
> | Phiên bản | Ngày | Nội dung |
> | --- | --- | --- |
> | 2.0.0 | 24/08/2026 | Gộp hai tài liệu rời thành một chuẩn duy nhất; bổ sung cài đặt tham chiếu cho `viRemoveDiacritics`, quy tắc tô sáng khi gõ không dấu, trần `page_size`, và kiểm tra quyền cho tham số lọc |
