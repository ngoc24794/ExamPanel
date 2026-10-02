# Quy Tắc Ràng Buộc & Tiêu Chí Tối Ưu

ExamPanel được xây dựng dựa trên mô hình toán học giải quyết bài toán phân công (Constraint Satisfaction & Combinatorial Optimization). Bài toán được chia thành hai nhóm quy tắc: **Ràng buộc cứng (Bắt buộc thỏa mãn)** và **Tiêu chí mềm (Tối ưu hóa chất lượng)**.

---

## 1. Ràng Buộc Cứng (Hard Constraints — H1 đến H7)

Đây là những điều kiện tiên quyết. Mọi phương án phân công hợp lệ **bắt buộc phải thỏa mãn 100%** các ràng buộc này. Nếu vi phạm dù chỉ 1 điều kiện, phương án sẽ bị coi là không hợp lệ và không thể đưa vào áp dụng.

| Mã Quy Tắc | Tên Quy Tắc | Diễn Giải Bằng Ngôn Ngữ Giáo Dục |
| :---: | :--- | :--- |
| **H1** | Không trùng lặp vai trò | Một giáo viên không thể vừa làm người ra đề vừa làm người phản biện cho cùng một ban đề trong cùng một kỳ thi. |
| **H2** | Đúng năng lực chuyên môn | Giáo viên chỉ được phân công ra đề hoặc phản biện cho những khối lớp mà Thầy/Cô có năng lực và được phân công giảng dạy trong năm học. |
| **H3** | Đa dạng phân hiệu ban đề | Trong mỗi ban đề (gồm 3 người: 2 người ra đề + 1 người phản biện), các thành viên phải đến từ **ít nhất 2 phân hiệu khác nhau** nhằm tránh tính cục bộ. |
| **H4** | Không quá tải cùng kỳ thi | Trong một kỳ thi cụ thể, mỗi giáo viên chỉ được tham gia tối đa 1 ban đề (không làm việc ở 2 khối lớp khác nhau cùng lúc). |
| **H5** | Tôn trọng lịch bận | Tuyệt đối không phân công giáo viên vào những kỳ thi mà giáo viên đã đăng ký lịch vắng (nghỉ ốm, công tác, con nhỏ). |
| **H6** | Tôn trọng khóa ghim & cấm | Tuân thủ các quyết định hành chính của Tổ trưởng: nếu đã ghim giáo viên A làm phản biện kỳ thi GK1 thì máy bắt buộc giữ nguyên; nếu đã cấm giáo viên B thì máy tuyệt đối không xếp. |
| **H7** | Khung chỉ tiêu khối lượng | Số lượt phân công cả năm của mỗi giáo viên phải nằm trong khoảng cho phép quanh chỉ tiêu lý thuyết $q$: $[\text{lo}, \text{hi}]$. Có 3 mức cấu hình dung sai (Tolerance):<br>• *Strict (0):* Nhận đúng chuẩn hoặc chênh lệch tối đa 0 lượt.<br>• *Standard (1 - Mặc định):* Dung sai $\pm 1$ lượt (ví dụ: chỉ tiêu 3.2 thì nhận từ 3 đến 4 lượt).<br>• *Flexible (2):* Dung sai $\pm 2$ lượt. |

---

## 2. Tiêu Chí Mềm (Soft Criteria — S1 đến S8)

Các tiêu chí này đại diện cho tính công bằng, sư phạm và sự hài hòa trong tổ chuyên môn. Thuật toán sẽ tìm phương án có **tổng điểm phạt (Penalty Score) thấp nhất**.

| Mã Tiêu Chí | Tên Tiêu Chí | Mục Đích Sư Phạm & Tổ Chức | Điểm Phạt |
| :---: | :--- | :--- | :---: |
| **S1** | Tần suất phản biện | Mỗi giáo viên nên phản biện từ 1 đến 2 lần trong năm để tích lũy kinh nghiệm đánh giá chuyên môn, không để ai bị "bỏ quên" vai trò phản biện. | Phạt nếu số lần phản biện $< 1$ hoặc $> 2$. |
| **S2** | Cân bằng tỷ lệ vai trò | Tỷ lệ số lần phản biện trên tổng số nhiệm vụ của giáo viên nên bám sát tỷ lệ chuẩn 1/3 (vì mỗi ban đề có 2 ra đề và 1 phản biện). | Phạt tỷ lệ lệch chuẩn. |
| **S3** | Phản biện độc lập phân hiệu | Khuyến khích người phản biện có phân hiệu khác với cả hai người ra đề. Điều này giúp nâng cao tính khách quan khi phản biện đề thi. | Phạt nếu người phản biện cùng phân hiệu với người ra đề. |
| **S4** | Đa dạng cặp ra đề | Tránh việc hai giáo viên cùng bắt cặp ra đề nhiều lần trong cùng một năm học. Luân chuyển đối tác giúp học hỏi lẫn nhau. | Phạt mỗi lần lặp lại cùng một cặp cộng sự. |
| **S5** | Tránh phản biện chéo lặp lại | Hạn chế tình trạng giáo viên A phản biện đề của giáo viên B lặp đi lặp lại ở nhiều kỳ thi. | Phạt lặp lại mối quan hệ phản biện. |
| **S6** | Giãn cách kỳ thi liên tiếp | Hạn chế việc một giáo viên phải ra đề ở hai kỳ thi liền kề nhau (ví dụ vừa làm GK1 lại vừa làm CK1) để giảm tải áp lực công việc. | Phạt hai kỳ thi ra đề liên tiếp. |
| **S7** | Luân chuyển khối lớp | Khuyến khích giáo viên dạy nhiều khối lớp được tham gia ban đề ở các khối khác nhau thay vì chỉ làm mãi một khối. | Phạt nếu chỉ tập trung vào một khối duy nhất. |
| **S8** | Cân bằng tải trọng thực tế | Đảm bảo khối lượng phân công bám sát hệ số tải trọng giảng dạy (Load Weight) của từng Thầy/Cô. | Phạt độ lệch $|n_i - q_i|$. |

---

## 3. Các Bộ Mẫu Cấu Hình Sẵn (Rule Presets)

Tại trang **Quy tắc & Tiêu chí** (Rules), Thầy/Cô có thể tùy ý điều chỉnh trọng số của từng tiêu chí mềm hoặc áp dụng nhanh các bộ mẫu cấu hình sẵn:

1. **Chuẩn mực (Balanced - Mặc định):** Cân bằng đồng đều giữa công bằng khối lượng, đa dạng đối tác và độc lập phân hiệu.
2. **Ưu tiên Công bằng Tải trọng (Fairness First):** Đặt trọng số cao nhất cho S8, S1, S2; đảm bảo số lượt phân công chuẩn xác từng cá nhân.
3. **Ưu tiên Khách quan Chuyên môn (Quality & Diversity):** Đặt trọng số cao nhất cho S3 (phản biện độc lập), S4 (đổi mới cặp ra đề) và S6 (giãn cách kỳ thi).
4. **Nhà trường Đa Phân hiệu (Multi-Campus Focus):** Tối ưu hóa việc giao lưu chuyên môn giữa các phân hiệu ở xa nhau.
