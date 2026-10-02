import * as React from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Printer, ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  usePlanDetails,
  useSchoolYears,
  useCampuses,
  useGrades,
  useExams,
  useTeachers,
  useSettings,
} from '@/lib/query/hooks'

export const PrintPlanPage: React.FC = () => {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const planId = Number(id)

  const { data: planDetails, isLoading } = usePlanDetails(planId)
  const { data: schoolYears = [] } = useSchoolYears()
  const { data: campuses = [] } = useCampuses()
  const { data: grades = [] } = useGrades()
  const schoolYear = schoolYears.find((y) => y.id === planDetails?.plan.school_year_id) || schoolYears[0]
  const { data: exams = [] } = useExams(schoolYear?.id)
  const { data: teachers = [] } = useTeachers(schoolYear?.id)
  const { data: settings } = useSettings()

  const teacherMap = React.useMemo(() => {
    const map = new Map<number, (typeof teachers)[0]['teacher']>()
    for (const tg of teachers) {
      map.set(tg.teacher.id, tg.teacher)
    }
    return map
  }, [teachers])

  const campusMap = React.useMemo(() => {
    const map = new Map<number, (typeof campuses)[0]>()
    for (const c of campuses) {
      map.set(c.id, c)
    }
    return map
  }, [campuses])

  if (isLoading || !planDetails) {
    return (
      <div className="p-8 text-center text-sm text-muted-foreground">
        Đang tải dữ liệu phân công...
      </div>
    )
  }

  const plan = planDetails.plan
  const assignments = planDetails.assignments
  const isDraft = !plan.is_final

  const schoolName = settings?.school_name || 'TRƯỜNG THPT CHUYÊN'
  const deptName = settings?.department_name || 'TỔ TOÁN - TIN'
  const signerTitle = settings?.signer_title || 'TỔ TRƯỞNG CHUYÊN MÔN'
  const signerName = settings?.signer_name || 'Nguyễn Văn A'
  const placeName = settings?.place_name || 'Hà Nội'

  const currentDate = new Date()
  const dateStr = `${placeName}, ngày ${currentDate.getDate()} tháng ${currentDate.getMonth() + 1} năm ${currentDate.getFullYear()}`

  return (
    <div className="min-h-screen bg-white text-black p-4 sm:p-8 font-sans print:p-0 print:m-0">
      <style>{`
        @page {
          size: A4 landscape;
          margin: 12mm;
        }
        @media print {
          .no-print {
            display: none !important;
          }
          body {
            background: white !important;
            color: black !important;
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
          }
          .page-container {
            width: 100% !important;
            max-width: none !important;
            padding: 0 !important;
            margin: 0 !important;
          }
        }
      `}</style>

      {/* Top Action Bar (hidden on print) */}
      <div className="no-print mb-6 max-w-6xl mx-auto flex items-center justify-between bg-neutral-100 p-4 rounded-lg border border-neutral-300">
        <Button
          variant="outline"
          size="sm"
          onClick={() => navigate('/assignments')}
          className="gap-2 text-xs"
        >
          <ArrowLeft className="h-4 w-4" />
          {t('print.backToApp')}
        </Button>
        <div className="flex items-center gap-3">
          <span className="text-xs text-neutral-600">
            {plan.name} {isDraft ? '(Bản nháp)' : '(Chính thức)'}
          </span>
          <Button
            data-testid="print-btn"
            size="sm"
            onClick={() => window.print()}
            className="gap-2 bg-blue-600 hover:bg-blue-700 text-white"
          >
            <Printer className="h-4 w-4" />
            {t('print.printBtn')}
          </Button>
        </div>
      </div>

      {/* Document Body */}
      <div className="page-container max-w-6xl mx-auto relative">
        {/* Draft Watermark */}
        {isDraft && (
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-0 opacity-10">
            <span className="text-8xl font-black tracking-widest text-red-600 rotate-[-25deg] select-none border-8 border-dashed border-red-600 p-8 rounded-2xl">
              {t('print.draftWatermark')}
            </span>
          </div>
        )}

        {/* Header Block */}
        <div className="grid grid-cols-2 gap-4 mb-6 relative z-10">
          <div className="text-center font-semibold text-xs leading-relaxed uppercase">
            <p>{schoolName}</p>
            <p className="font-bold underline">{deptName}</p>
          </div>
          <div className="text-center font-semibold text-xs leading-relaxed">
            <p className="uppercase font-bold">CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM</p>
            <p className="font-medium underline italic">Độc lập - Tự do - Hạnh phúc</p>
          </div>
        </div>

        {/* Title */}
        <div className="text-center mb-6 relative z-10">
          <h1 className="text-lg font-bold uppercase tracking-wide">
            {isDraft && '[BẢN NHÁP] '}BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA
          </h1>
          <p className="text-xs italic mt-1 font-medium">
            Năm học: {schoolYear?.name} — Phương án: {plan.name}
          </p>
        </div>

        {/* Matrix Table */}
        <table className="w-full border-collapse border border-neutral-400 text-xs mb-8 relative z-10">
          <thead>
            <tr className="bg-neutral-100 font-bold text-center">
              <th className="border border-neutral-400 p-2 w-28">Kỳ thi</th>
              <th className="border border-neutral-400 p-2 w-20">Khối</th>
              <th className="border border-neutral-400 p-2">Cán bộ ra đề 1</th>
              <th className="border border-neutral-400 p-2">Cán bộ ra đề 2</th>
              <th className="border border-neutral-400 p-2">Cán bộ phản biện</th>
            </tr>
          </thead>
          <tbody>
            {exams.map((exam) => {
              const examGrades = grades
              return examGrades.map((grade, gradeIdx) => {
                const setters = assignments.filter(
                  (a) =>
                    a.exam_id === exam.id &&
                    a.grade_id === grade.id &&
                    a.role === 'setter',
                )
                const reviewer = assignments.find(
                  (a) =>
                    a.exam_id === exam.id &&
                    a.grade_id === grade.id &&
                    a.role === 'reviewer',
                )

                const s1 = setters[0] ? teacherMap.get(setters[0].teacher_id) : null
                const s2 = setters[1] ? teacherMap.get(setters[1].teacher_id) : null
                const rev = reviewer ? teacherMap.get(reviewer.teacher_id) : null

                const s1Campus = s1 ? campusMap.get(s1.campus_id)?.name : ''
                const s2Campus = s2 ? campusMap.get(s2.campus_id)?.name : ''
                const revCampus = rev ? campusMap.get(rev.campus_id)?.name : ''

                return (
                  <tr key={`${exam.id}-${grade.id}`} className="hover:bg-neutral-50">
                    {gradeIdx === 0 && (
                      <td
                        rowSpan={examGrades.length}
                        className="border border-neutral-400 p-2 font-bold text-center align-middle bg-neutral-50/50"
                      >
                        {exam.name}
                        <div className="text-[10px] text-neutral-500 font-mono">({exam.code})</div>
                      </td>
                    )}
                    <td className="border border-neutral-400 p-2 font-semibold text-center align-middle">
                      {grade.name}
                    </td>
                    <td className="border border-neutral-400 p-2">
                      {s1 ? (
                        <div>
                          <span className="font-semibold">{s1.full_name}</span>
                          <span className="text-[11px] text-neutral-600 block">
                            {s1.code ? `[${s1.code}] ` : ''}{s1Campus}
                          </span>
                        </div>
                      ) : (
                        <span className="text-neutral-400">-</span>
                      )}
                    </td>
                    <td className="border border-neutral-400 p-2">
                      {s2 ? (
                        <div>
                          <span className="font-semibold">{s2.full_name}</span>
                          <span className="text-[11px] text-neutral-600 block">
                            {s2.code ? `[${s2.code}] ` : ''}{s2Campus}
                          </span>
                        </div>
                      ) : (
                        <span className="text-neutral-400">-</span>
                      )}
                    </td>
                    <td className="border border-neutral-400 p-2">
                      {rev ? (
                        <div>
                          <span className="font-semibold">{rev.full_name}</span>
                          <span className="text-[11px] text-neutral-600 block">
                            {rev.code ? `[${rev.code}] ` : ''}{revCampus}
                          </span>
                        </div>
                      ) : (
                        <span className="text-neutral-400">-</span>
                      )}
                    </td>
                  </tr>
                )
              })
            })}
          </tbody>
        </table>

        {/* Signature Block */}
        <div className="grid grid-cols-2 gap-8 text-xs relative z-10 break-inside-avoid">
          <div>
            <p className="italic text-neutral-500 mb-1">
              * Lưu ý: Mọi cán bộ được phân công có trách nhiệm bảo mật đề thi và thực hiện đúng tiến độ.
            </p>
          </div>
          <div className="text-center font-medium">
            <p className="italic mb-2">{dateStr}</p>
            <p className="font-bold uppercase mb-16">{signerTitle}</p>
            <p className="font-bold">{signerName}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
