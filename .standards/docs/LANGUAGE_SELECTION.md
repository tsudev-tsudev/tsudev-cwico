# CHỌN NGÔN NGỮ VÀ NỀN TẢNG THEO LOẠI PROJECT (v2.0.0)

> **Bắt buộc** với mọi project/tool/phần mềm trong hệ sinh thái tsudev, áp dụng
> cho cả lập trình viên và agent AI. Muốn đi ra ngoài bảng này thì theo quy trình
> ngoại lệ ở mục 7, không tự quyết.
>
> Ký hiệu mức độ: `MUST` = bắt buộc, `SHOULD` = nên, `MAY` = tùy chọn.

## Mục lục

- [1. Sáu nguyên tắc](#1-sáu-nguyên-tắc)
- [2. Bảng tra cứu nhanh](#2-bảng-tra-cứu-nhanh)
- [3. Website và ứng dụng web](#3-website-và-ứng-dụng-web)
- [4. Ứng dụng di động](#4-ứng-dụng-di-động)
- [5. Phần mềm desktop và công cụ](#5-phần-mềm-desktop-và-công-cụ)
- [6. Quy trình trước khi khởi tạo project](#6-quy-trình-trước-khi-khởi-tạo-project)
- [7. Quy trình ngoại lệ](#7-quy-trình-ngoại-lệ)
- [8. Yêu cầu riêng cho agent AI](#8-yêu-cầu-riêng-cho-agent-ai)
- [9. Checklist tuân thủ](#9-checklist-tuân-thủ)

**Ký hiệu mức ưu tiên trong các bảng:**

| Ký hiệu | Ý nghĩa |
| --- | --- |
| ⭐ | Lựa chọn mặc định. Chọn cái này nếu không có lý do khác |
| ✅ | Được chấp nhận, phù hợp trong ngữ cảnh đã nêu |
| ⚠️ | Chỉ khi có lý do kỹ thuật hoặc nghiệp vụ rõ ràng, `MUST` ghi lý do vào `docs/ARCHITECTURE.md` |
| ⛔ | Không dùng cho loại project này |

---

## 1. Sáu nguyên tắc

1. **Đúng công cụ, đúng việc.** `MUST NOT` chọn ngôn ngữ chỉ vì quen tay. Chọn
   theo đặc tính kỹ thuật của project: hiệu năng, bảo mật, hệ sinh thái thư
   viện, khả năng bảo trì dài hạn.
2. **Nhất quán trong hệ sinh thái.** Project cùng loại `SHOULD` dùng chung stack
   đã chuẩn hóa ở mục 2. Mỗi lần lệch là một lần chi phí học lại cho người sau.
3. **Minh bạch quyết định.** Mọi lựa chọn ngôn ngữ/framework `MUST` được ghi vào
   `docs/ARCHITECTURE.md` (mục "Tech Stack") kèm lý do và các phương án đã cân nhắc.
4. **Ưu tiên khả năng bảo trì.** Project có vòng đời dự kiến trên 3 tháng hoặc
   có từ 2 người tham gia `MUST` dùng ngôn ngữ có hệ thống kiểu rõ ràng
   (TypeScript thay vì JavaScript thuần).
5. **Bảo mật theo mặc định.** Hệ thống xử lý dữ liệu mức D2/D3 theo
   [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 2 `MUST` ưu tiên ngôn ngữ
   và framework có hệ sinh thái bảo mật trưởng thành, thay vì ưu tiên tốc độ
   phát triển ban đầu.
6. **Đọc được bởi cả người và máy.** Cấu trúc project, tên biến, comment và tài
   liệu `MUST` viết sao cho cả lập trình viên lẫn agent AI hiểu ngay, không phải
   suy đoán ngữ cảnh ẩn nằm ngoài file.

**Ràng buộc bắt buộc thứ bảy:** mọi lựa chọn `MUST` chạy được trong hạn mức 0
đồng theo [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md). Một stack tuyệt vời nhưng
chỉ chạy được trên hạ tầng trả phí là stack sai cho hệ sinh thái này.

---

## 2. Bảng tra cứu nhanh

### 2.1. Website và ứng dụng web

| Thành phần | Ngôn ngữ / Framework | Mức | Ghi chú |
| --- | --- | --- | --- |
| Frontend | TypeScript + React / Next.js | ⭐ | Mặc định cho mọi frontend mới |
| Frontend | TypeScript + Vue.js | ✅ | Khi đội đã có kinh nghiệm Vue |
| Frontend | JavaScript thuần | ⚠️ | Chỉ cho trang tĩnh hoặc trang giới thiệu nhỏ. `MUST` chuyển sang TypeScript khi project mở rộng |
| Backend - AI/dữ liệu | Python (FastAPI) | ⭐ | Web có tích hợp AI, học máy, xử lý dữ liệu |
| Backend - AI/dữ liệu | Python (Django) | ✅ | Khi cần sẵn trang quản trị và ORM mạnh |
| Backend - doanh nghiệp/tài chính | Java (Spring Boot) | ⭐ | Hệ thống cần độ tin cậy và kiểm soát lỗi tại thời điểm biên dịch |
| Backend - doanh nghiệp/tài chính | C# (.NET) | ✅ | Tương đương Java; ưu tiên nếu đã ở hệ sinh thái Microsoft |
| Backend - nội dung/bán hàng vừa và nhỏ | PHP (Laravel) | ✅ | Blog, tin tức, thương mại điện tử quy mô vừa |
| Backend - nội dung/bán hàng vừa và nhỏ | WordPress (PHP) | ⚠️ | Chỉ khi cần CMS dựng nhanh, ít tùy biến logic. `MUST` có kế hoạch vá bảo mật định kỳ - đây là nền tảng bị tấn công nhiều nhất |
| Backend - hiệu năng cao/thời gian thực | Go | ✅ | API tốc độ cao, dịch vụ nhỏ, WebSocket |
| Backend - hiệu năng cao/thời gian thực | Node.js (TypeScript) | ✅ | Khi muốn dùng chung kiểu dữ liệu với frontend |
| Hàm biên (edge function) | TypeScript trên Cloudflare Workers | ⭐ | Mặc định cho API nhẹ. Xem `FREE_TIER_STACK.md` |

### 2.2. Ứng dụng di động

| Thành phần | Ngôn ngữ / Framework | Mức | Ghi chú |
| --- | --- | --- | --- |
| iOS gốc | Swift | ⭐ | Khi cần hiệu năng và trải nghiệm nền tảng tối đa |
| Android gốc | Kotlin | ⭐ | Ngôn ngữ được Google khuyến nghị chính thức |
| Android gốc | Java | ⚠️ | Chỉ khi bảo trì mã cũ. `MUST NOT` dùng cho project mới |
| Đa nền tảng | Dart (Flutter) | ⭐ | Mặc định cho đa nền tảng, một mã nguồn, hiệu năng gần với gốc |
| Đa nền tảng | TypeScript (React Native) | ✅ | Khi đội đã mạnh về Web/React |

### 2.3. Desktop và công cụ

| Thành phần | Ngôn ngữ / Framework | Mức | Ghi chú |
| --- | --- | --- | --- |
| Ứng dụng Windows/doanh nghiệp | C# (.NET MAUI / WPF) | ⭐ | Tối ưu cho hệ sinh thái Windows |
| Đồ họa, kỹ thuật, game engine | C++ | ⭐ | Cần truy cập phần cứng trực tiếp, hiệu năng tối đa |
| Đồ họa, kỹ thuật, game engine | Rust | ⭐ | Thay C++ khi an toàn bộ nhớ là yêu cầu quan trọng |
| Công cụ tiện ích, tự động hóa | Python | ⭐ | Script nhanh, hệ sinh thái thư viện lớn |
| Desktop đa nền tảng | TypeScript (Electron) | ✅ | Khi cần tái dùng kỹ năng và mã Web |
| Desktop đa nền tảng | TypeScript (Tauri) | ✅ | Khi Electron quá nặng - bản build nhỏ hơn nhiều, lõi Rust |
| Desktop đa nền tảng | Dart (Flutter Desktop) | ✅ | Khi đã dùng Flutter cho di động và muốn dùng chung mã |

---

## 3. Website và ứng dụng web

**Frontend.** TypeScript là mặc định bắt buộc cho mọi frontend từ mức trung bình
trở lên. JavaScript thuần chỉ chấp nhận cho trang tĩnh đơn giản. Chọn framework:

- **React** khi cần hệ sinh thái lớn nhất và nhiều người biết làm.
- **Next.js** khi cần dựng sẵn ở máy chủ (SSR) hoặc SEO tốt.
- **Vue.js** khi đội đã quen, hoặc cần đường học ngắn hơn.

**Backend.** Chọn theo bản chất nghiệp vụ, không theo sở thích:

- **Python (FastAPI/Django)** khi backend cần tích hợp AI/học máy, xử lý dữ liệu
  lớn, hoặc cần đi nhanh với hệ sinh thái thư viện khoa học dữ liệu.
- **Java/C#** cho ngân hàng, tài chính, doanh nghiệp lớn - nơi bảo mật, bắt lỗi
  tại thời điểm biên dịch và độ ổn định dài hạn quan trọng hơn tốc độ khởi động
  dự án.
- **PHP (Laravel)** cho trang tin tức, blog, thương mại điện tử vừa và nhỏ, nhất
  là khi cần triển khai nhanh trên hạ tầng lưu trữ phổ thông.
- **Go** khi cần thông lượng cao với mức tiêu thụ bộ nhớ thấp.

**Ràng buộc bắt buộc cho mọi lựa chọn web:**

- `MUST` áp dụng đủ header bảo mật tại [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 7.2.
- `MUST` đạt WCAG 2.1 AA theo [`ACCESSIBILITY.md`](ACCESSIBILITY.md).
- `MUST` dùng token giao diện, cấm hard-code màu và cỡ chữ.
- Có chức năng soạn thảo nội dung: `MUST` theo [`RICH_TEXT_EDITOR.md`](RICH_TEXT_EDITOR.md).
- Có chức năng tìm kiếm/lọc: `MUST` theo [`SEARCH_AND_FILTER.md`](SEARCH_AND_FILTER.md).

---

## 4. Ứng dụng di động

**Ứng dụng gốc (native)** - chọn khi hiệu năng và trải nghiệm nền tảng là ưu
tiên cao nhất:

- **iOS**: Swift.
- **Android**: Kotlin.

**Đa nền tảng** - chọn khi cần tối ưu chi phí và thời gian trên cả hai nền tảng:

- **Dart + Flutter**: mặc định. Giao diện nhất quán, hiệu năng gần với gốc, một
  mã nguồn duy nhất.
- **TypeScript + React Native**: khi đội đã có nền tảng Web/React và muốn tái
  dùng kiến thức.

**Ràng buộc bắt buộc:**

- `MUST NOT` lưu token hay khóa API trong mã nguồn ứng dụng - mọi ứng dụng di
  động đều dịch ngược được.
- `MUST` dùng kho lưu trữ an toàn của hệ điều hành (Keychain trên iOS,
  EncryptedSharedPreferences trên Android) cho dữ liệu nhạy cảm.
- `MUST` ghim chứng chỉ (certificate pinning) cho ứng dụng xử lý dữ liệu D3.
- Khóa ký ứng dụng `MUST NOT` nằm trong repo - xem nhóm 2 của
  [`GITIGNORE_POLICY.md`](GITIGNORE_POLICY.md).

---

## 5. Phần mềm desktop và công cụ

- **Ứng dụng Windows/doanh nghiệp**: **C#** với **.NET MAUI** hoặc **WPF**.
- **Công cụ đồ họa, kỹ thuật, game engine**: **C++** và **Rust**. Rust được ưu
  tiên khi an toàn bộ nhớ là yêu cầu quan trọng - phần lớn lỗ hổng nghiêm trọng
  trong phần mềm hệ thống là lỗi quản lý bộ nhớ.
- **Công cụ tiện ích và tự động hóa**: **Python**.
- **Desktop đa nền tảng**: **Electron** khi cần tái dùng mã Web; **Tauri** khi
  kích thước bản cài và mức tiêu thụ bộ nhớ quan trọng.

**Ràng buộc bắt buộc:**

- Tên file phát hành `MUST` theo khuôn tại [`VERSIONING.md`](VERSIONING.md) mục 2.
- `MUST` công bố SHA-256 của mỗi file cài để người dùng kiểm chứng.
- Ứng dụng có cơ chế tự cập nhật `MUST` xác minh chữ ký của bản cập nhật trước
  khi cài. Kênh cập nhật không xác minh là đường thẳng để phát tán mã độc.
- Electron `MUST` bật `contextIsolation`, tắt `nodeIntegration` ở tiến trình
  render, và không bao giờ nạp nội dung từ xa vào cửa sổ có đặc quyền.

---

## 6. Quy trình trước khi khởi tạo project

Mọi project mới `MUST` đi hết 7 bước sau **trước khi** viết dòng mã đầu tiên:

1. **Xác định loại project**: Website / Di động / Desktop-Công cụ.
2. **Xác định ràng buộc nghiệp vụ**: dữ liệu ở mức phân loại nào? có yêu cầu
   thời gian thực không? đội hiện có kỹ năng gì? vòng đời dự kiến bao lâu?
3. **Chọn ngôn ngữ/framework** theo bảng mục 2, ưu tiên mức ⭐.
4. **Đối chiếu hạn mức 0 đồng** theo [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md)
   mục 4 - trả lời đủ 6 câu hỏi ở đó.
5. **Ghi quyết định** vào `docs/ARCHITECTURE.md`: đã chọn gì, vì sao, đã cân
   nhắc phương án nào khác.
6. **Khởi tạo cấu trúc chuẩn** theo [`PROJECT_STRUCTURE.md`](PROJECT_STRUCTURE.md)
   và checklist khởi tạo tại [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 10.1.
7. **Rà soát bởi ít nhất một người hoặc agent khác** trước khi merge cấu trúc
   khởi tạo vào nhánh chính.

---

## 7. Quy trình ngoại lệ

Khi project cần ngôn ngữ/framework nằm ngoài bảng (mức ⛔ hoặc chưa được liệt kê):

1. Người đề xuất `MUST` viết mục **"Lý do ngoại lệ"** trong `docs/ARCHITECTURE.md`,
   nêu rõ:
   - Vì sao các lựa chọn ⭐/✅ không đáp ứng được.
   - Rủi ro và cách giảm thiểu: ai bảo trì được, tài liệu ra sao, nếu người đó
     nghỉ thì sao.
   - Ràng buộc 0 đồng có còn giữ được không.
2. Ngoại lệ `MUST` được ghi nhận qua Pull Request vào thư mục `exceptions/` của
   repo `tsudev-standards`, để làm tiền lệ tham chiếu cho project sau.
3. Nếu cùng một ngoại lệ được chấp nhận từ **3 lần trở lên** cho cùng một loại
   nhu cầu, `MUST` mở đề xuất cập nhật chính thức bảng ở mục 2. Ngoại lệ lặp lại
   ba lần không còn là ngoại lệ - đó là quy ước đang thiếu.

---

## 8. Yêu cầu riêng cho agent AI

Agent AI được giao khởi tạo hoặc phát triển project `MUST`:

1. **Đọc hết tài liệu này** trước khi đề xuất hay chọn ngôn ngữ/framework cho bất
   kỳ project nào trong hệ sinh thái tsudev.
2. **Không tự ý bỏ qua lựa chọn ⭐** khi người dùng không nêu ràng buộc khác.
   Cần dùng mức ✅ hoặc ⚠️ thì `MUST` giải thích lý do trước khi thực hiện.
3. **Luôn ghi lựa chọn** vào `docs/ARCHITECTURE.md`, kể cả khi được yêu cầu
   "làm nhanh". Làm nhanh rút gọn phạm vi, không rút gọn việc ghi lại quyết định.
4. **Không đề xuất thứ nằm ngoài bảng** trừ khi tài liệu chưa bao phủ nhu cầu đó.
   Khi phải làm vậy, `MUST` nói rõ đây là **đề xuất bổ sung, chưa phải quy ước**,
   và khuyến nghị mở PR cập nhật tài liệu này.
5. **Giữ tính nhất quán trong project đang có.** Nếu project đã dùng ngôn ngữ
   khác với khuyến nghị hiện tại, `MUST NOT` tự đề xuất viết lại toàn bộ. Chỉ áp
   quy ước này cho phần mã mới, hoặc khi được yêu cầu refactor rõ ràng.
6. **Giữ mã, comment, tên biến ở trạng thái đọc được bởi cả người và máy**: rõ
   ràng, không viết tắt mơ hồ, không phụ thuộc ngữ cảnh ẩn nằm ngoài file.
7. **Đối chiếu ràng buộc 0 đồng** mỗi khi đề xuất một dịch vụ hoặc thư viện có
   phần trả phí.

---

## 9. Checklist tuân thủ

Dùng trước khi khởi tạo hoặc rà soát một project mới:

- [ ] Đã xác định đúng loại project (Website / Di động / Desktop-Công cụ).
- [ ] Đã tra bảng mục 2 và chọn ngôn ngữ/framework phù hợp.
- [ ] Lựa chọn đã ghi vào `docs/ARCHITECTURE.md` kèm lý do và phương án thay thế.
- [ ] Đã đối chiếu hạn mức 0 đồng và ghi mục "Hạn mức đang dùng".
- [ ] Đã phân loại dữ liệu theo `SECURITY_BASELINE.md` mục 2.
- [ ] Nếu dùng mức ⚠️ hoặc ngoại lệ: đã hoàn thành quy trình mục 7.
- [ ] Cấu trúc project theo `PROJECT_STRUCTURE.md`.
- [ ] Đã hoàn thành checklist khởi tạo repo tại `SECURITY_BASELINE.md` mục 10.1.
- [ ] Đã qua rà soát của ít nhất một người hoặc agent khác.

---

## 10. Từ khóa tra cứu

`ngon-ngu-lap-trinh` `lua-chon-cong-nghe` `tech-stack` `website` `web-app`
`frontend` `backend` `mobile-app` `ios` `android` `swift` `kotlin` `flutter`
`dart` `react-native` `desktop-app` `electron` `tauri` `tools` `automation`
`python` `typescript` `javascript` `java` `csharp` `php` `cpp` `rust` `go`
`quy-uoc-trung-tam` `tsudev-standards` `ai-agent-guideline`
