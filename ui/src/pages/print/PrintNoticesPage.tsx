import * as React from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Printer, ArrowLeft, Filter } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from '@/components/ui/select'
import {
  usePlanDetails,
  useSchoolYears,
  useCampuses,
  useGrades,
  useExams,
  useTeachers,
  useSettings,
} from '@/lib/query/hooks'

export const PrintNoticesPage: React.FC = () => {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const planId = Number(id)

  const [filterMode, setFilterMode] = React.useState<'assigned' | 'all'>('assigned')

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

  const examMap = React.useMemo(() => {
    const map = new Map<number, (typeof exams)[0]>()
    for (const e of exams) {
      map.set(e.id, e)
    }
    return map
  }, [exams])

  const gradeMap = React.useMemo(() => {
    const map = new Map<number, (typeof grades)[0]>()
    for (const g of grades) {
      map.set(g.id, g)
    }
    return map
  }, [grades])

  if (isLoading || !planDetails) {
    return (
      <div className="p-8 text-center text-sm text-muted-foreground">
        Đang tải dữ liệu giấy báo...
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

  // Group assignments per teacher
  const assignmentsByTeacher = new Map<number, typeof assignments>()
  for (const a of assignments) {
    if (!assignmentsByTeacher.has(a.teacher_id)) {
      assignmentsByTeacher.set(a.teacher_id, [])
    }
    assignmentsByTeacher.get(a.teacher_id)!.push(a)
  }

  // Filter teachers to render
  const eligibleTeachers = teachers.filter((tg) => {
    if (filterMode === 'assigned') {
      return (assignmentsByTeacher.get(tg.teacher.id) || []).length > 0
    }
    return true
  })

  return (
    <div className="min-h-screen bg-white text-black p-4 sm:p-8 font-sans print:p-0 print:m-0">
      <style>{`
        @page {
          size: A4 portrait;
          margin: 15mm;
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
          .notice-page {
            page-break-after: always;
            break-after: page;
            min-height: 95vh;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
          }
          .notice-page:last-child {
            page-break-after: avoid;
            break-after: avoid;
          }
        }
        @media screen {
          .notice-page {
            margin-bottom: 2rem;
            padding-bottom: 2rem;
            border-bottom: 2px dashed #ccc;
          }
        }
      `}</style>

      {/* Top Action Bar (hidden on print) */}
      <div className="no-print mb-6 max-w-4xl mx-auto flex flex-wrap items-center justify-between gap-4 bg-neutral-100 p-4 rounded-lg border border-neutral-300">
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
          {/* Filter */}
          <div className="flex items-center gap-2">
            <Filter className="h-4 w-4 text-neutral-500" />
            <Select
              value={filterMode}
              onValueChange={(val: 'assigned' | 'all') => setFilterMode(val)}
            >
              <SelectTrigger className="w-[200px] h-8 text-xs bg-white">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="assigned">{t('print.filterOnlyAssigned')}</SelectItem>
                <SelectItem value="all">{t('print.filterAll')}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <span className="text-xs font-medium text-neutral-600">
            ({eligibleTeachers.length} giấy báo)
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

      {/* Notices Container */}
      <div className="max-w-4xl mx-auto">
        {eligibleTeachers.map((tg) => {
          const teacher = tg.teacher
          const teacherAssignments = assignmentsByTeacher.get(teacher.id) || []
          const campus = campusMap.get(teacher.campus_id)

          return (
            <div key={teacher.id} className="notice-page relative">
              {/* Draft Watermark */}
              {isDraft && (
                <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-0 opacity-10">
                  <span className="text-8xl font-black tracking-widest text-red-600 rotate-[-25deg] select-none border-8 border-dashed border-red-600 p-8 rounded-2xl">
                    {t('print.draftWatermark')}
                  </span>
                </div>
              )}

              <div className="relative z-10">
                {/* Header */}
                <div className="grid grid-cols-2 gap-4 mb-6">
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
                <div className="text-center mb-6">
                  <h1 className="text-base font-bold uppercase tracking-wide">
                    {isDraft && '[BẢN NHÁP] '}THÔNG BÁO PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA
                  </h1>
                  <p className="text-xs italic mt-1 font-medium">
                    Năm học: {schoolYear?.name}
                  </p>
                </div>

                {/* Teacher Info */}
                <div className="bg-neutral-50 p-4 rounded border border-neutral-300 mb-6 text-xs leading-relaxed">
                  <p>
                    <span className="font-semibold">{t('print.teacherLabel')}</span>{' '}
                    <span className="font-bold text-sm text-neutral-900">{teacher.full_name}</span>
                    {teacher.code && (
                      <span className="ml-2 font-mono text-neutral-600">[{teacher.code}]</span>
                    )}
                  </p>
                  <p className="mt-1">
                    <span className="font-semibold">{t('print.campusLabel')}</span>{' '}
                    <span>{campus?.name || '-'}</span>
                  </p>
                </div>

                {/* Tasks Table */}
                <div className="mb-6">
                  <p className="text-xs font-bold mb-2 uppercase text-neutral-800">
                    {t('print.taskTable')}
                  </p>
                  {teacherAssignments.length === 0 ? (
                    <p className="text-xs italic text-neutral-500 py-3">
                      {t('print.noTasks')}
                    </p>
                  ) : (
                    <table className="w-full border-collapse border border-neutral-400 text-xs">
                      <thead>
                        <tr className="bg-neutral-100 font-bold text-center">
                          <th className="border border-neutral-400 p-2 w-12">STT</th>
                          <th className="border border-neutral-400 p-2 w-36">Kỳ thi</th>
                          <th className="border border-neutral-400 p-2 w-20">Khối</th>
                          <th className="border border-neutral-400 p-2 w-28">Vai trò</th>
                          <th className="border border-neutral-400 p-2">Thành viên cùng ban đề</th>
                        </tr>
                      </thead>
                      <tbody>
                        {teacherAssignments.map((task, idx) => {
                          const exam = examMap.get(task.exam_id)
                          const grade = gradeMap.get(task.grade_id)
                          const roleLabel =
                            task.role === 'setter'
                              ? t('print.roleSetter')
                              : t('print.roleReviewer')

                          // Find co-panelists in this slot
                          const panelAssignments = assignments.filter(
                            (a) =>
                              a.exam_id === task.exam_id &&
                              a.grade_id === task.grade_id &&
                              a.teacher_id !== teacher.id,
                          )

                          const coPanelists = panelAssignments
                            .map((pa) => {
                              const tObj = teacherMap.get(pa.teacher_id)
                              const cObj = tObj ? campusMap.get(tObj.campus_id) : null
                              const roleTag = pa.role === 'setter' ? 'Ra đề' : 'Phản biện'
                              return `${tObj?.full_name || 'N/A'} (${roleTag} - ${cObj?.name || ''})`
                            })
                            .join('; ')

                          return (
                            <tr key={idx} className="hover:bg-neutral-50">
                              <td className="border border-neutral-400 p-2 text-center font-mono">
                                {idx + 1}
                              </td>
                              <td className="border border-neutral-400 p-2 font-semibold">
                                {exam?.name || `Kỳ thi #${task.exam_id}`}
                              </td>
                              <td className="border border-neutral-400 p-2 text-center font-semibold">
                                {grade?.name || `Khối ${task.grade_id}`}
                              </td>
                              <td className="border border-neutral-400 p-2 text-center font-bold">
                                {roleLabel}
                              </td>
                              <td className="border border-neutral-400 p-2 text-neutral-700 leading-normal">
                                {coPanelists || '-'}
                              </td>
                            </tr>
                          )
                        })}
                      </tbody>
                    </table>
                  )}
                </div>

                {/* Confidential Notice */}
                <p className="text-[11px] italic text-neutral-600 mb-8">
                  {t('print.confidentialNotice')}
                </p>
              </div>

              {/* Signature Block */}
              <div className="grid grid-cols-2 gap-8 text-xs relative z-10 break-inside-avoid mt-8">
                <div />
                <div className="text-center font-medium">
                  <p className="italic mb-2">{dateStr}</p>
                  <p className="font-bold uppercase mb-16">{signerTitle}</p>
                  <p className="font-bold">{signerName}</p>
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
