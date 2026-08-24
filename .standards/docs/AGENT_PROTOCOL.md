# GIAO THỨC LÀM VIỆC CỦA AGENT (v2.0.0)

> Tài liệu này mô tả **cách vận hành một phiên làm việc** cho cả lập trình viên
> và agent AI: mở phiên, khóa file, bàn giao, đóng phiên. Quy ước nội dung nằm ở
> các tài liệu khác; đây là quy trình.

## 1. Ba file điều phối

| File | Vai trò | Ai ghi |
| --- | --- | --- |
| `logs/STATE.md` | Hàng đợi việc, việc đang làm, việc đã xong, quyết định quan trọng | Mọi phiên, đầu và cuối |
| `logs/LOCKS.md` | File nào đang bị ai khóa | Người/agent trước khi sửa file |
| `logs/handover/` | Phiếu bàn giao giữa các phiên | Phiên sắp kết thúc |

Cả ba `MUST` được commit vào repo. Chúng là bộ nhớ chung của dự án, không phải
file tạm.

## 2. Mở phiên

Dán nguyên khối này vào đầu **mọi** phiên terminal mới:

```
Đọc AGENTS.md, logs/STATE.md và phiếu bàn giao mới nhất trong logs/handover/.
Tuân thủ toàn bộ quy ước. Nhận task tiếp theo trong hàng đợi của STATE.md,
khóa file mình sẽ sửa vào logs/LOCKS.md rồi mới bắt đầu. Trả lời ngắn gọn,
tiết kiệm token, không lặp lại nội dung đã có trong file quy ước.
```

Agent `MUST` thực hiện đúng thứ tự: đọc quy ước -> đọc trạng thái -> đọc phiếu
bàn giao -> nhận việc -> khóa file -> mới bắt đầu sửa.

## 3. Khóa file (chống giẫm chân)

Trước khi sửa **bất kỳ** file nào:

1. Đọc `logs/LOCKS.md`.
2. File chưa ai khóa: thêm dòng theo đúng định dạng, rồi mới sửa.
   ```
   src/features/search/index.ts | agent-2 / tìm kiếm không dấu | 14:30 24/08/2026
   ```
3. Sửa xong: **xóa dòng khóa của mình ngay**, không đợi cuối phiên.

**File đang bị người khác khóa: TUYỆT ĐỐI không sửa.** Thay vào đó tạo phiếu bàn
giao tại `logs/handover/` theo mẫu `docs/templates/HANDOVER.md`, ghi rõ cần thay
đổi gì, vì sao, và tiêu chí thế nào là xong.

Người đang giữ khóa `MUST` đọc phiếu gửi đến mình **trước khi** nhả khóa, thực
hiện hoặc từ chối có lý do, và ghi kết quả vào chính phiếu đó.

`MUST NOT` can thiệp vào nhiệm vụ, nhánh git, hay file của người khác đang làm.

## 4. Tiết kiệm token và ngữ cảnh

- Không đọc lại file đã nắm nội dung trong cùng phiên.
- Không in toàn bộ file dài ra hội thoại - chỉ trích phần liên quan.
- Trả lời và ghi log **ngắn gọn, gạch đầu dòng, không văn mẫu**.
- Tri thức dùng lại được (quyết định kiến trúc, cách chạy build, lỗi đã gặp) ghi
  vào file markdown tương ứng **một lần duy nhất**; các phiên sau chỉ tham chiếu
  đường dẫn.
- Task lớn `MUST` chia nhỏ. Làm xong phần nào chốt phần đó vào log ngay, để mất
  phiên không mất công.

## 5. Đóng phiên và bàn giao

Kích hoạt khi: (a) hàng đợi trong `STATE.md` cạn, (b) được yêu cầu bàn giao,
hoặc (c) ngữ cảnh sắp cạn.

1. Ghi vào `logs/STATE.md`: việc đã làm, việc dang dở **kèm bước tiếp theo cụ
   thể**, quyết định quan trọng.
2. Tạo phiếu `logs/handover/YYYYMMDD-NN_<chủ-đề>.md` theo mẫu. Tiêu chuẩn: phiên
   sau đọc xong làm tiếp được **không cần hỏi lại**, kể cả khi máy tắt đột ngột.
3. Nhả toàn bộ khóa của mình trong `logs/LOCKS.md`.
4. Dọn tàn dư: xóa file tạm, không để thay đổi chưa commit nằm ngoài phạm vi bàn
   giao. Cây làm việc sạch để phiên sau không tốn "chi phí chết".
5. Đề xuất chủ project đóng terminal và mở phiên mới cho task kế tiếp.

## 6. Ranh giới của agent AI

Ngoài các mục ở `SECURITY_BASELINE.md` mục 12, agent `MUST NOT`:

- Sửa file quy ước (`AGENTS.md`, `.standards/**`, `docs/*` được đánh dấu KHÔNG
  SỬA, `tokens/*`, `.gitignore`) trừ khi chủ project yêu cầu trực tiếp.
- Tự ý `git push`, mở PR, hay merge khi chưa được yêu cầu.
- Tự ý xóa file hay thư mục ngoài phạm vi task.
- Chạy lệnh phá hủy (`rm -rf`, `git reset --hard`, `DROP TABLE`) mà không hỏi trước.
- Thêm phụ thuộc mới mà không khai báo theo `SECURITY_BASELINE.md` mục 5.

Và `MUST`:

- Báo cáo trung thực: test trượt thì nói là trượt kèm output; bỏ qua bước nào thì
  nói rõ bước đó.
- Khi phát hiện quy ước tự mâu thuẫn hoặc sai, báo cho chủ project và đề xuất qua
  `proposals/`, không tự sửa theo ý mình.

## 7. Nhiều agent làm song song

- Mỗi agent nhận **một nhiệm vụ duy nhất** tại một thời điểm, ghi trong
  `STATE.md` mục "Đang thực hiện".
- Mỗi agent làm trên **nhánh riêng**.
- Phân chia theo **ranh giới file**, không theo ranh giới tính năng - vì hai
  tính năng có thể đụng cùng một file.
- Khi hai nhiệm vụ buộc phải đụng cùng file: một agent làm, agent kia gửi phiếu
  bàn giao. Không có ngoại lệ.
