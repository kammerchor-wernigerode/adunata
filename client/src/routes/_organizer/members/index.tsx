import { createFileRoute, Link } from '@tanstack/react-router'
import { useId, useState } from 'react'
import { getListMembersQueryOptions, useListMembers } from '@/api/generated/api'
import type { Member } from '@/api/generated/model'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useTranslate } from '@/l10n/translate'

export const Route = createFileRoute('/_organizer/members/')({
  loader: ({ context: { queryClient } }) => queryClient.query(getListMembersQueryOptions()),
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
        <Table className="overflow-hidden rounded-lg border border-border">
          <TableHeader className="bg-muted text-xs">
            <TableRow className="hover:bg-transparent">
              <TableHead className="px-3 font-semibold text-muted-foreground">{translate('member-name')}</TableHead>
              <TableHead className="px-3 font-semibold text-muted-foreground">{translate('member-mail')}</TableHead>
              <TableHead className="px-3 font-semibold text-muted-foreground">{translate('member-status')}</TableHead>
              <TableHead className="px-3 font-semibold text-muted-foreground">{translate('member-role')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {shown.map((member) => (
              <TableRow key={member.id} className="even:bg-muted/50">
                <TableCell className="px-3 py-2">
                  <Link
                    to="/members/$memberId"
                    params={{ memberId: String(member.id) }}
                    className="font-semibold text-foreground underline underline-offset-4"
                  >
                    {member.lastName}, {member.firstName}
                  </Link>
                </TableCell>
                <TableCell className="px-3 py-2 break-all whitespace-normal">{member.mail}</TableCell>
                <TableCell className="px-3 py-2">{translate(member.active ? 'member-active' : 'member-inactive')}</TableCell>
                <TableCell className="px-3 py-2">{member.organizer ? translate('member-organizer') : ''}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </>
  )
}
