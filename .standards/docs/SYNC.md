# ĐỒNG BỘ BỘ QUY ƯỚC - repo con luôn ở trạng thái mới nhất (v2.0.0)

> Quy ước chỉ có giá trị khi mọi repo thực sự đang chạy cùng một bản. Tài liệu
> này mô tả cơ chế bắt buộc để repo con biết mình đang ở bản nào, và biết ngay
> khi mình đã lệch.

## 1. Mô hình

```
                tsudev-standards  (Public, nguồn chân lý)
                        |
                        |  scripts/sync-standards.sh  (không cần token)
                        v
    tsudev-web/.standards/   swico/.standards/   <repo khác>/.standards/
        (bản sao chỉ-đọc, ĐƯỢC COMMIT vào repo con)
```

**Một chiều duy nhất.** Sửa quy ước thì sửa ở trung tâm rồi đồng bộ xuống. Không
bao giờ sửa `.standards/` ở repo con - mọi thay đổi ở đó sẽ bị lần đồng bộ sau
ghi đè, và cổng kiểm sẽ báo lệch.

Muốn đổi quy ước: mở đề xuất tại `proposals/` hoặc Issue ở repo trung tâm. Xem
[`../CONTRIBUTING.md`](../CONTRIBUTING.md).

## 2. Vì sao `.standards/` phải được commit

Có ba lý do, cả ba đều là lý do vận hành thật:

1. **Đọc được khi không có mạng.** Lập trình viên và agent AI phải mở được quy
   ước ngay trong repo đang làm việc, không phải đi tìm ở nơi khác.
2. **CI kiểm được mà không gọi ra ngoài.** Cổng kiểm chạy trên bản đã commit,
   nhanh và không phụ thuộc GitHub còn sống hay không.
3. **Truy nguyên được.** Nhìn lịch sử git của repo con là biết ngày nào nó nâng
   lên bản quy ước nào, và code lúc đó tuân bản nào.

## 3. Cài đặt lần đầu cho một repo con

```bash
# 1. Lấy script đồng bộ (một lần duy nhất)
mkdir -p scripts
curl -fsSL https://raw.githubusercontent.com/tsudev-tsudev/tsudev-standards/main/scripts/sync-standards.sh \
  -o scripts/sync-standards.sh
chmod +x scripts/sync-standards.sh

# 2. Đồng bộ
./scripts/sync-standards.sh

# 3. Ghép .gitignore theo GITIGNORE_POLICY.md mục 2
cat .standards/templates/gitignore/base.gitignore > .gitignore
cat .standards/templates/gitignore/node.gitignore >> .gitignore   # đúng ngôn ngữ của repo

# 4. Dựng thư mục phối hợp giữa các phiên
# LƯU Ý: file trong .standards/ là chỉ-đọc, và `cp` giữ nguyên quyền đó.
# Không mở lại quyền ghi thì agent không cập nhật được STATE.md ở cuối phiên.
mkdir -p logs/handover
cp .standards/templates/logs/STATE.md logs/STATE.md
cp .standards/templates/logs/LOCKS.md logs/LOCKS.md
chmod u+w logs/STATE.md logs/LOCKS.md
touch logs/handover/.gitkeep

# 5. Lấy script cổng kiểm
curl -fsSL https://raw.githubusercontent.com/tsudev-tsudev/tsudev-standards/main/scripts/check-standards.sh \
  -o scripts/check-standards.sh
chmod +x scripts/check-standards.sh

# 6. Commit toàn bộ
git add .standards .standards-version .gitignore logs scripts
git commit -m "chore(standards): đồng bộ bộ quy ước tsudev v2.0.0"
```

Sau bước này, repo con `MUST` có `AGENTS.md` ở gốc trỏ về `.standards/AGENTS.md`
và bổ sung phần phân vai riêng. Mẫu: `.standards/templates/AGENTS.downstream.md`.

### 3.1. Loại `.standards/` khỏi mọi công cụ tự sửa file

`MUST` thêm `.standards/` vào danh sách loại trừ của **mọi** công cụ có khả năng
ghi đè file: Prettier, ESLint `--fix`, Black, `dotnet format`, `clang-format`,
`rustfmt`, và cả cấu hình "format on save" của IDE.

```bash
printf '\n# Bản sao chỉ-đọc của bộ quy ước trung tâm - không định dạng lại.\n.standards/\n' >> .prettierignore
printf '\n# Bản sao chỉ-đọc của bộ quy ước trung tâm - không lint, không sửa.\n.standards/\n' >> .eslintignore
```

**Vì sao bắt buộc:** `MANIFEST.sha256` băm từng byte của mỗi file quy ước. Một
lần Prettier chạy qua là đủ đổi dấu cách và xuống dòng, làm lệch băm, và
`./scripts/sync-standards.sh --check` sẽ báo lệch **vĩnh viễn** cho tới khi ai
đó đồng bộ lại. Tệ hơn: cổng kiểm CI đỏ vì một lý do không liên quan gì tới nội
dung công việc, và người ta bắt đầu học cách bỏ qua nó.

Đây là lỗi có thật, gặp ngay ở repo đầu tiên áp bộ quy ước v2.

## 4. Cập nhật định kỳ

```bash
./scripts/sync-standards.sh              # lấy bản mới nhất của nhánh main
./scripts/sync-standards.sh --ref v2.1.0 # hoặc ghim theo nhãn phát hành
git add .standards .standards-version
git commit -m "chore(standards): nâng lên quy ước v2.1.0"
```

`MUST` đọc `CHANGELOG.md` của bản mới trước khi commit. Nếu là thay đổi phá vỡ
(số chính tăng), `MUST` thực hiện phần "Hướng dẫn nâng cấp" trong CHANGELOG
trước khi merge.

## 5. Phát hiện lệch tự động

### 5.1. Trên CI của repo con (bắt buộc)

Chặn merge khi repo con đã lệch bản quy ước:

```yaml
# .github/workflows/standards.yml
name: Cổng kiểm quy ước
on: [pull_request, push]

permissions:
  contents: read

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
      - name: Kiểm quy ước cục bộ
        run: ./scripts/check-standards.sh
      - name: Đối chiếu với bộ quy ước trung tâm
        run: ./scripts/sync-standards.sh --check
```

### 5.2. Tự mở PR nâng cấp hằng tuần (khuyến nghị)

```yaml
# .github/workflows/standards-update.yml
name: Nâng cấp bộ quy ước
on:
  schedule:
    - cron: '0 1 * * 1'   # 08:00 giờ Việt Nam, thứ Hai hằng tuần
  workflow_dispatch:

permissions:
  contents: write
  pull-requests: write

jobs:
  bump:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
      - run: ./scripts/sync-standards.sh
      - uses: peter-evans/create-pull-request@271a8d0340265f705b14b6d32b9829c1cb33d45e # v7.0.8
        with:
          branch: chore/standards-bump
          title: 'chore(standards): nâng bộ quy ước lên bản mới nhất'
          body: |
            PR tự động. Đọc CHANGELOG của bộ quy ước trước khi merge.
            Nếu là thay đổi phá vỡ, làm phần "Hướng dẫn nâng cấp" trước.
          commit-message: 'chore(standards): đồng bộ bộ quy ước'
          delete-branch: true
```

## 6. Bảo đảm toàn vẹn

Mỗi bản phát hành của bộ quy ước đi kèm `MANIFEST.sha256` liệt kê SHA-256 của
từng file. `sync-standards.sh` **tự xác minh** bản tải về trước khi ghi vào
`.standards/`, và từ chối nếu không khớp.

Kiểm lại bằng tay bất cứ lúc nào:

```bash
cd .standards && sha256sum --check MANIFEST.sha256
```

Nếu có file lệch, ai đó đã sửa bản sao chỉ-đọc. Cách xử lý đúng: hoàn nguyên bản
sao (`./scripts/sync-standards.sh`), rồi đưa thay đổi đó lên trung tâm dưới dạng
đề xuất.

## 7. Ghim bản cho môi trường sản xuất

Repo phục vụ người dùng thật `SHOULD` ghim theo nhãn phát hành thay vì `main`:

```bash
./scripts/sync-standards.sh --ref v2.0.0
```

Ghi lựa chọn này vào `.env.example` qua biến `TSUDEV_STANDARDS_REF` để người sau
biết repo cố ý đứng ở bản nào, chứ không phải quên nâng.

## 8. Xử lý sự cố

| Hiện tượng | Nguyên nhân thường gặp | Cách xử lý |
| --- | --- | --- |
| `Không giải nén được bản tải về` | Sai `--ref`, hoặc mạng chặn codeload | Kiểm tra nhãn có tồn tại; thử lại với `--ref main` |
| `bản tải về không khớp MANIFEST.sha256` | Bản trung tâm quên chạy `make-manifest.sh`, hoặc gói bị can thiệp trên đường truyền | **Không dùng bản đó.** Báo ngay theo `SECURITY.md` mục 3 |
| `LỆCH: .standards/... khác bản trung tâm` | Có người sửa bản sao chỉ-đọc | Chạy `./scripts/sync-standards.sh`, đưa thay đổi lên `proposals/` |
| `thiếu .standards-version` | Chép tay thay vì chạy script | Chạy `./scripts/sync-standards.sh` |
| Không ghi được vào `.standards/` | File đã bị đặt chỉ-đọc sau khi đồng bộ | Đúng như thiết kế. Đừng `chmod` để sửa, hãy sửa ở trung tâm |
| Cổng kiểm CI đỏ ngay sau khi chạy công cụ định dạng | Prettier hoặc tương đương đã sửa file trong `.standards/`, làm lệch băm | Thêm `.standards/` vào `.prettierignore` và `.eslintignore` theo mục 3.1, rồi `./scripts/sync-standards.sh` để khôi phục |
| `Permission denied` khi agent ghi `logs/STATE.md` | File được `cp` từ `.standards/templates/` nên thừa hưởng quyền chỉ-đọc | `chmod u+w logs/STATE.md logs/LOCKS.md` |
| `git checkout` báo `unable to unlink '.standards/...'` | Cây cũ đồng bộ bằng bản trước v2.2.1 bị đặt chỉ-đọc cả **thư mục** | `chmod -R u+w .standards` một lần, rồi `./scripts/sync-standards.sh` để lấy bản đã vá |
