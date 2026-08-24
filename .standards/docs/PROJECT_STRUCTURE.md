# CẤU TRÚC THƯ MỤC CHUẨN - mọi repo trong hệ sinh thái tuân theo (v2.0.0)

Mục tiêu: bất kỳ lập trình viên hoặc agent AI nào mở repo đều biết ngay file,
biến, hàm nằm ở đâu - không phải suy đoán.

Quy ước nhận **hai hình trạng**. Repo tự khai mình theo hình trạng nào trong
`AGENTS.md` phần B. Mọi quy tắc ở mục 3 áp dụng cho **cả hai**.

---

## 1. Hình trạng A - ứng dụng đơn

Dùng cho: một app web, một app Electron, một CLI, một thư viện, một dịch vụ.

```
<ten-project>/
├── AGENTS.md                  # Điểm vào bắt buộc. Trỏ về .standards/ + phần riêng của repo
├── README.md                  # Giới thiệu ngắn + cách chạy trong 5 phút
├── SECURITY.md                # Chính sách bảo mật + kênh báo lỗ hổng
├── CHANGELOG.md               # Theo docs/VERSIONING.md mục 3
├── .gitignore                 # base + phần theo ngôn ngữ + phần riêng (KHÔNG xóa bớt base)
├── .gitattributes             # Chuẩn hóa kết thúc dòng
├── .editorconfig
├── .env.example               # Mẫu biến môi trường - KHÔNG chứa giá trị thật
├── .standards-version         # Sinh bởi script - repo đang ở bản quy ước nào
│
├── .standards/                # BẢN SAO CHỈ-ĐỌC của bộ quy ước (được commit, KHÔNG SỬA)
│   ├── AGENTS.md
│   ├── docs/
│   ├── tokens/
│   └── templates/
│
├── .github/
│   ├── workflows/             # CI: cổng kiểm quy ước, bảo mật, lint, test
│   ├── dependabot.yml
│   └── pull_request_template.md
│
├── docs/                      # Tài liệu CỦA RIÊNG repo này (chỉ markdown)
│   ├── ARCHITECTURE.md        # Quyết định kiến trúc, tech stack, phân loại dữ liệu, hạn mức đang dùng
│   └── adr/                   # Quyết định lớn, mỗi cái một file theo mẫu ADR
│
├── logs/                      # Trạng thái phối hợp giữa các phiên (ĐƯỢC COMMIT)
│   ├── STATE.md               # Hàng đợi task + đang làm + đã xong + quyết định
│   ├── LOCKS.md               # File đang bị khóa: path | agent | thời điểm
│   └── handover/              # Phiếu bàn giao: YYYYMMDD-NN_<chu-de>.md
│
├── src/                       # TOÀN BỘ mã nguồn
│   ├── main/  hoặc  app/      # Điểm vào ứng dụng
│   ├── components/            # Component giao diện tái sử dụng (mỗi cái một thư mục)
│   ├── features/              # Mỗi tính năng một thư mục: giao diện + logic + test đi cùng
│   ├── services/              # Gọi API, cơ sở dữ liệu, hệ thống file
│   ├── utils/                 # Hàm thuần tái sử dụng (không tác dụng phụ)
│   ├── styles/                # Style toàn cục - chỉ nhập từ tokens/, cấm định nghĩa màu mới
│   └── types/  hoặc  models/  # Kiểu dữ liệu dùng chung
│
├── assets/                    # brand/ (BRAND_ASSETS.md), icon/, fonts/, images/
├── tests/                     # Test tích hợp và E2E (unit test đặt cạnh mã trong features/)
├── scripts/                   # Script build, phát hành, đồng bộ (đặt tên theo động từ)
│   ├── sync-standards.sh
│   └── check-standards.sh
└── dist/  build/  release/    # Sản phẩm build - LUÔN nằm trong .gitignore
```

---

## 2. Hình trạng B - monorepo nhiều workspace

Dùng khi repo khai `workspaces` trong `package.json` (hoặc cơ chế tương đương ở
hệ khác), và các đơn vị bên trong có **ranh giới build hoặc ranh giới tiến trình
riêng**.

```
<ten-project>/
├── AGENTS.md, README.md, SECURITY.md, CHANGELOG.md, .gitignore, ...
├── .standards/                # Bản sao quy ước, đặt ở GỐC repo, dùng chung
├── docs/                      # Tài liệu dùng chung toàn repo
├── logs/                      # Điều phối phiên, dùng chung toàn repo
├── scripts/                   # Script dùng chung toàn repo
│
├── apps/                      # Ứng dụng chạy được, mỗi cái một workspace
│   └── frontend-main/
│       └── src/               # Hình trạng A thu nhỏ bên trong
│
├── services/                  # TIẾN TRÌNH ĐỘC LẬP, mỗi cái một workspace
│   ├── content-service/
│   │   └── src/
│   └── auth-service/
│       └── src/
│
└── packages/                  # Thư viện dùng chung, có ranh giới build riêng
    ├── ui/
    │   └── src/
    └── types/
        └── src/
```

**Khác biệt duy nhất so với hình trạng A** là **nơi** cây `src/` sống:

- **Không có `src/` ở gốc repo.** Gốc chỉ giữ thứ dùng chung: `docs/`, `logs/`,
  `.standards/`, `scripts/`, cấu hình build và CI.
- Mỗi workspace là một hình trạng A thu nhỏ: `<workspace>/src/...`.
- Quy tắc "hàm dùng chung từ 2 nơi trở lên phải gom lại" được thỏa bằng một
  package dùng chung (`packages/<tên>/`) thay vì `src/utils/`.

**Phân biệt `services/` với `src/services/`** - đây là chỗ hay nhầm:

| | Là gì | Ở đâu |
| --- | --- | --- |
| `services/` (gốc repo, hình trạng B) | **Tiến trình độc lập**, tự chạy, tự triển khai | Chỉ có ở monorepo |
| `src/services/` (trong một workspace) | **Thư mục mã** gọi API, cơ sở dữ liệu, hệ thống file | Có ở cả hai hình trạng |

Ranh giới sở hữu và quyền sửa file giữa các workspace `MUST` khai ở `AGENTS.md`
phần B.

---

## 3. Quy tắc áp dụng cho cả hai hình trạng

### 3.1. Đặt tên

- **Thư mục và file thường**: `kebab-case` (`user-profile/`, `date-format.ts`).
- **File markdown quy ước**: `UPPER_SNAKE.md` (`ARCHITECTURE.md`, `SECURITY.md`).
- **Biến và hàm**: `camelCase`, tên nói rõ việc (`formatDateVN()`, `loadTokens()`).
- **Hằng số**: `UPPER_SNAKE_CASE`.
- **Class và component**: `PascalCase`.
- **Hàm xử lý tiếng Việt**: tiền tố `vi` (`viNormalizeText()`) theo
  [`SEARCH_AND_FILTER.md`](SEARCH_AND_FILTER.md) mục 3.2.
- C#, C++, Java, Go theo chuẩn của ngôn ngữ, nhưng giữ nguyên ngữ nghĩa tên.
- `MUST NOT` viết tắt mơ hồ: `d`, `tmp2`, `handleIt`, `data2`.

### 3.2. Tổ chức mã

- **Mỗi file một trách nhiệm.** File vượt **400 dòng** `MUST` được cân nhắc tách.
- Hàm dùng chung ở **từ 2 nơi trở lên** `MUST` chuyển vào `src/utils/`,
  `src/services/`, hoặc package dùng chung. `MUST NOT` copy-paste.
- Mọi giá trị màu, cỡ chữ, khoảng cách `MUST` truy ngược được về
  `tokens/design-tokens.json`.

### 3.3. Vị trí bắt buộc

| Thứ này | Luôn nằm ở |
| --- | --- |
| Bộ quy ước | `.standards/` ở gốc repo, chỉ-đọc |
| Điều phối phiên | `logs/` ở gốc repo, được commit |
| Tài liệu riêng của repo | `docs/` ở gốc repo |
| Sản phẩm build | Trong `.gitignore`, không bao giờ commit |
| Secret | `.env` cục bộ hoặc kho secret của nền tảng - không bao giờ trong repo |

---

## 4. Bộ quy ước và token trung tâm

Bộ `docs/` chuẩn, `tokens/` và `templates/` đặt tại repo trung tâm
**`tsudev-tsudev/tsudev-standards`** (Public). Repo con lấy về bằng
`scripts/sync-standards.sh` - xem [`SYNC.md`](SYNC.md).

Đổi một mã màu ở trung tâm, chạy đồng bộ, toàn hệ sinh thái cập nhật. `MUST NOT`
chép tay file quy ước giữa các repo con.
