import * as React from 'react'
import { api } from '@/lib/api'
import { useParams, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Printer, ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  usePlanDetails,
  useSchoolYears,
  useGrades,
  useExams,
  useTeachers,
  useSubjects,
  useSettings,
} from '@/lib/query/hooks'

export const PrintPlanPage: React.FC = () => {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const planId = Number(id)

  const { data: planDetails, isLoading } = usePlanDetails(planId)
  const { data: schoolYears = [] } = useSchoolYears()
  const { data: grades = [] } = useGrades()
  const schoolYear =
    schoolYears.find((y) => y.id === planDetails?.plan.school_year_id) || schoolYears[0]
  const { data: exams = [] } = useExams(schoolYear?.id)
  const { data: teachers = [] } = useTeachers(schoolYear?.id)
  const { data: subjects = [] } = useSubjects(schoolYear?.id)
  const { data: settings } = useSettings()

  const teacherMap = React.useMemo(() => {
    const map = new Map<number, (typeof teachers)[0]['teacher']>()
    for (const tg of teachers) {
      map.set(tg.teacher.id, tg.teacher)
    }
    return map
  }, [teachers])

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

  const effectiveSubjects =
    subjects.length > 0
      ? subjects
      : [{ id: 1, name: 'Chung', code: 'CHUNG', setters: 2, reviewers: 1 }]
  const maxSetters = Math.max(...effectiveSubjects.map((s) => s.setters || 1), 1)
  const maxReviewers = Math.max(...effectiveSubjects.map((s) => s.reviewers || 1), 1)

  const teacherTotals = teachers.map((tg) => {
    const t = tg.teacher
    const tAssignments = assignments.filter((a) => a.teacher_id === t.id)
    const total = tAssignments.length
    const de = tAssignments.filter((a) => a.role === 'setter').length
    const pb = tAssignments.filter((a) => a.role === 'reviewer').length
    const perExam = exams.map(
      (e) => tAssignments.filter((a) => a.exam_id === e.id).length,
    )
    return {
      teacher: t,
      displayName: t.display_name?.trim() || t.full_name,
      total,
      de,
      pb,
      perExam,
    }
  })

  const grandTotal = teacherTotals.reduce((sum, item) => sum + item.total, 0)
  const grandDe = teacherTotals.reduce((sum, item) => sum + item.de, 0)
  const grandPb = teacherTotals.reduce((sum, item) => sum + item.pb, 0)
  const grandPerExam = exams.map((_, eIdx) =>
    teacherTotals.reduce((sum, item) => sum + (item.perExam[eIdx] || 0), 0),
  )

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
            onClick={() => void api.printPage()}
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

        {/* Q-Style Matrix & Attached Totals Table */}
        <div className="overflow-x-auto mb-8 relative z-10">
          <table className="w-full border-collapse border border-neutral-700 text-[11px] leading-tight text-center">
            <thead>
              <tr className="bg-neutral-200 font-bold">
                <th
                  rowSpan={effectiveSubjects.length > 1 ? 2 : 1}
                  colSpan={2}
                  className="border border-neutral-600 p-1.5 w-24"
                >
                  Kì thi/khối
                </th>
                {grades.map((grade) => (
                  <th
                    key={grade.id}
                    colSpan={effectiveSubjects.length > 1 ? effectiveSubjects.length : 1}
                    className="border border-neutral-600 p-1.5"
                  >
                    {grade.name}
                  </th>
                ))}
                <th
                  rowSpan={effectiveSubjects.length > 1 ? 2 : 1}
                  className="w-2 border-y-0 border-neutral-300 bg-white p-0"
                ></th>
                <th
                  rowSpan={effectiveSubjects.length > 1 ? 2 : 1}
                  className="border border-neutral-600 p-1.5 w-24 text-left"
                >
                  GV
                </th>
                <th
                  rowSpan={effectiveSubjects.length > 1 ? 2 : 1}
                  className="border border-neutral-600 p-1.5 w-16"
                >
                  Tổng lượt n.vụ
                </th>
                <th
                  rowSpan={effectiveSubjects.length > 1 ? 2 : 1}
                  className="border border-neutral-600 p-1.5 w-10"
                >
                  Đề
                </th>
                <th
                  rowSpan={effectiveSubjects.length > 1 ? 2 : 1}
                  className="border border-neutral-600 p-1.5 w-10"
                >
                  PB
                </th>
                {exams.map((exam) => (
                  <th
                    key={exam.id}
                    rowSpan={effectiveSubjects.length > 1 ? 2 : 1}
                    className="border border-neutral-600 p-1.5 w-10"
                  >
                    {exam.code}
                  </th>
                ))}
              </tr>
              {effectiveSubjects.length > 1 && (
                <tr className="bg-neutral-100 font-semibold">
                  {grades.map((grade) =>
                    effectiveSubjects.map((sub) => (
                      <th
                        key={`${grade.id}-${sub.id}`}
                        className="border border-neutral-600 p-1"
                      >
                        {sub.code}
                      </th>
                    )),
                  )}
                </tr>
              )}
            </thead>
            <tbody>
              {(() => {
                const examRows = exams.flatMap((exam, examIdx) => {
                  const rows = []
                  const bgClass = examIdx % 2 === 0 ? 'bg-white' : 'bg-neutral-50/70'

                  // Setter rows
                  for (let k = 0; k < maxSetters; k++) {
                    rows.push({
                      exam,
                      examIdx,
                      isFirstInExam: k === 0,
                      totalExamRows: maxSetters + maxReviewers,
                      roleLabel: 'Đề',
                      role: 'setter' as const,
                      position: k,
                      bgClass,
                    })
                  }
                  // Reviewer rows
                  for (let m = 0; m < maxReviewers; m++) {
                    rows.push({
                      exam,
                      examIdx,
                      isFirstInExam: false,
                      totalExamRows: maxSetters + maxReviewers,
                      roleLabel: 'P.Biện',
                      role: 'reviewer' as const,
                      position: m,
                      bgClass,
                    })
                  }
                  return rows
                })

                const totalRows = Math.max(examRows.length, teacherTotals.length + 1)
                const rows = []

                for (let r = 0; r < totalRows; r++) {
                  const gridRow = examRows[r]
                  const teacherRow = teacherTotals[r]
                  const isGrandTotalRow = r === teacherTotals.length

                  rows.push(
                    <tr key={`row-${r}`} className={gridRow?.bgClass || 'bg-white'}>
                      {/* Grid: Exam cell */}
                      {gridRow
                        ? gridRow.isFirstInExam && (
                            <td
                              rowSpan={gridRow.totalExamRows}
                              className="border border-neutral-600 p-1 font-bold text-center align-middle bg-neutral-100"
                            >
                              {gridRow.exam.code}
                            </td>
                          )
                        : r >= examRows.length &&
                          r === examRows.length && (
                            <td
                              colSpan={2 + grades.length * effectiveSubjects.length}
                              rowSpan={totalRows - examRows.length}
                              className="border border-neutral-300 bg-neutral-50/30"
                            ></td>
                          )}

                      {/* Grid: Role label & Subject cells */}
                      {gridRow && (
                        <>
                          <td className="border border-neutral-600 p-1 font-semibold text-center align-middle bg-neutral-50">
                            {gridRow.roleLabel}
                          </td>
                          {grades.map((grade) =>
                            effectiveSubjects.map((sub) => {
                              const canFit =
                                gridRow.role === 'setter'
                                  ? gridRow.position < (sub.setters || 1)
                                  : gridRow.position < (sub.reviewers || 1)

                              if (!canFit) {
                                return (
                                  <td
                                    key={`${gridRow.exam.id}-${grade.id}-${sub.id}-${gridRow.position}`}
                                    className="border border-neutral-600 p-1 bg-neutral-100 text-neutral-300"
                                  >
                                    -
                                  </td>
                                )
                              }

                              const assignment = assignments.find(
                                (a) =>
                                  a.exam_id === gridRow.exam.id &&
                                  a.grade_id === grade.id &&
                                  a.subject_id === sub.id &&
                                  a.role === gridRow.role &&
                                  a.position === gridRow.position,
                              )
                              const t = assignment
                                ? teacherMap.get(assignment.teacher_id)
                                : null
                              const displayName = t
                                ? t.display_name?.trim() || t.full_name
                                : '-'

                              return (
                                <td
                                  key={`${gridRow.exam.id}-${grade.id}-${sub.id}-${gridRow.position}`}
                                  className="border border-neutral-600 p-1 font-medium text-center align-middle"
                                >
                                  {displayName}
                                </td>
                              )
                            }),
                          )}
                        </>
                      )}

                      {/* Blank spacer column */}
                      <td className="w-2 border-y-0 border-neutral-300 bg-white p-0"></td>

                      {/* Totals Table */}
                      {teacherRow ? (
                        <>
                          <td className="border border-neutral-600 p-1 text-left font-medium">
                            {teacherRow.displayName}
                          </td>
                          <td className="border border-neutral-600 p-1 font-bold">
                            {teacherRow.total}
                          </td>
                          <td className="border border-neutral-600 p-1">
                            {teacherRow.de}
                          </td>
                          <td className="border border-neutral-600 p-1">
                            {teacherRow.pb}
                          </td>
                          {exams.map((exam, eIdx) => (
                            <td key={exam.id} className="border border-neutral-600 p-1">
                              {teacherRow.perExam[eIdx] || 0}
                            </td>
                          ))}
                        </>
                      ) : isGrandTotalRow ? (
                        <>
                          <td className="border border-neutral-600 p-1 text-left font-bold bg-neutral-200">
                            Tổng cộng
                          </td>
                          <td className="border border-neutral-600 p-1 font-bold bg-neutral-200">
                            {grandTotal}
                          </td>
                          <td className="border border-neutral-600 p-1 font-bold bg-neutral-200">
                            {grandDe}
                          </td>
                          <td className="border border-neutral-600 p-1 font-bold bg-neutral-200">
                            {grandPb}
                          </td>
                          {exams.map((exam, eIdx) => (
                            <td
                              key={exam.id}
                              className="border border-neutral-600 p-1 font-bold bg-neutral-200"
                            >
                              {grandPerExam[eIdx]}
                            </td>
                          ))}
                        </>
                      ) : (
                        <td
                          colSpan={4 + exams.length}
                          className="border border-neutral-300 bg-neutral-50/20"
                        ></td>
                      )}
                    </tr>,
                  )
                }

                return rows
              })()}
            </tbody>
          </table>
        </div>

        {/* Signature Block */}
        <div className="grid grid-cols-2 gap-8 text-xs relative z-10 break-inside-avoid">
          <div>
            <p className="italic text-neutral-500 mb-1">
              * Lưu ý: Mọi cán bộ được phân công có trách nhiệm bảo mật đề thi và thực
              hiện đúng tiến độ.
            </p>
          </div>
          <div className="text-center font-medium ml-auto w-72">
            <p className="italic mb-2">{dateStr}</p>
            <p className="font-bold uppercase mb-16">{signerTitle}</p>
            <p className="font-bold">{signerName}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
