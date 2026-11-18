import { createFileRoute, Link } from '@tanstack/react-router'
import { useId, useState } from 'react'
import { getListMembersQueryOptions, useListMembers } from '@/api/generated/api'
import type { Member } from '@/api/generated/model'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useTranslate } from '@/l10n/translate'

export const Route = createFileRoute('/_organizer/members/')({
  loader: ({ context: { queryClient } }) => queryClient.ensureQueryData(getListMembersQueryOptions()),
  component: MemberListPage,
})

type Filter = 'all' | 'active' | 'inactive'

function matches(member: Member, search: string, filter: Filter): boolean {
  const needle = search.trim().toLocaleLowerCase()
  const text = `${member.firstName} ${member.lastName} ${member.mail}`.toLocaleLowerCase()
  const status = filter === 'all' || (filter === 'active') === member.active
  return status && text.includes(needle)
}

function MemberListPage() {
  const id = useId()
  const translate = useTranslate()
  const [search, setSearch] = useState('')
  const [filter, setFilter] = useState<Filter>('all')
  const members = useListMembers()

  const shown = members.data?.status === 200 ? members.data.data.filter((member) => matches(member, search, filter)) : []

  return (
    <>
      <PageTitle title={translate('members-title')} />
      <div className="mb-4 flex flex-wrap items-end justify-between gap-4">
        <Field className="max-w-sm">
          <FieldLabel htmlFor={`${id}-search`}>{translate('members-search')}</FieldLabel>
          <Input id={`${id}-search`} type="search" value={search} onChange={(event) => setSearch(event.target.value)} />
        </Field>
        <Button asChild>
          <Link to="/members/new">{translate('members-add')}</Link>
        </Button>
      </div>
      <ToggleGroup
        type="single"
        value={filter}
        onValueChange={(value) => value && setFilter(value as Filter)}
        aria-label={translate('members-filter')}
        className="mb-3"
      >
        {(['all', 'active', 'inactive'] as const).map((option) => (
          <ToggleGroupItem key={option} value={option}>
            {translate(`members-filter-${option}`)}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      <p role="status" className="mb-2 text-sm text-muted-foreground">
        {members.data ? translate('members-count', { count: shown.length }) : ''}
      </p>
      {members.data && shown.length === 0 ? (
        <p>{translate('members-empty')}</p>
      ) : (
        <table className="w-full border-collapse overflow-hidden rounded-lg border border-border text-sm">
          <thead className="bg-muted text-left text-xs text-muted-foreground">
            <tr>
              <th className="px-3 py-2 font-semibold">{translate('member-name')}</th>
              <th className="px-3 py-2 font-semibold">{translate('member-mail')}</th>
              <th className="px-3 py-2 font-semibold">{translate('member-status')}</th>
              <th className="px-3 py-2 font-semibold">{translate('member-role')}</th>
            </tr>
          </thead>
          <tbody>
            {shown.map((member) => (
              <tr key={member.id} className="border-t border-border even:bg-muted/50">
                <td className="px-3 py-2">
                  <Link
                    to="/members/$memberId"
                    params={{ memberId: String(member.id) }}
                    className="font-semibold text-foreground underline underline-offset-4"
                  >
                    {member.lastName}, {member.firstName}
                  </Link>
                </td>
                <td className="px-3 py-2 break-all">{member.mail}</td>
                <td className="px-3 py-2">{translate(member.active ? 'member-active' : 'member-inactive')}</td>
                <td className="px-3 py-2">{member.organizer ? translate('member-organizer') : ''}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  )
}
