! Input/lifecycle adapter only. Called by a generated copy of the upstream entry.
module matterviz_session
use iso_c_binding
use iso_fortran_env, only: input_unit,output_unit
implicit none
logical :: session_enabled=.false.
interface
 integer(c_int) function mwfn_session_initialize() bind(C)
 import c_int
 end function
 integer(c_int) function mwfn_session_managed() bind(C)
 import c_int
 end function
 integer(c_int) function mwfn_session_active() bind(C)
 import c_int
 end function
 integer(c_int64_t) function mwfn_session_end() bind(C)
 import c_int64_t
 end function
 integer(c_int) function mwfn_session_absolute(file,capacity) bind(C)
 import c_int,c_char
 character(c_char) :: file(*)
 integer(c_int),value :: capacity
 end function
 integer(c_int) function mwfn_session_menu(pos,file,length,silent,cores,rejected) bind(C)
 import c_int,c_int64_t,c_char
 integer(c_int64_t),value :: pos
 character(c_char) :: file(*)
 integer(c_int),value :: length,rejected
 integer(c_int) :: silent,cores
 end function
end interface
contains
subroutine matterviz_session_initialize()
integer :: fd,istat
character(len=80) :: path
fd=mwfn_session_initialize()
if (fd<0) return
write(path,'(a,i0)') '/proc/self/fd/',fd
close(input_unit)
open(unit=input_unit,file=trim(path),access='stream',form='formatted',status='old',iostat=istat)
if (istat/=0) error stop 'Cannot attach managed input stream'
session_enabled=.true.
end subroutine

subroutine matterviz_session_read_main(command)
use defvar, only: filename,isilent,nthreads
character(len=*),intent(out) :: command
character(len=4096) :: discarded
character(c_char),allocatable :: file(:)
integer(c_int64_t) :: position
integer(c_int) :: action,rejected
integer :: i,length
if (.not.session_enabled) then
 read(*,*) command
 return
end if
rejected=0
do
 flush(output_unit)
 inquire(unit=input_unit,pos=position)
 allocate(file(len(filename)+1))
 file=c_null_char
 do i=1,len_trim(filename)
  file(i)=filename(i:i)
 end do
 length=mwfn_session_absolute(file,int(size(file),c_int))
 filename=''
 do i=1,length
  filename(i:i)=file(i)
 end do
 action=mwfn_session_menu(position,file,int(len_trim(filename),c_int),isilent,nthreads,rejected)
 deallocate(file)
 if (action==1) then
  command='0'
  return
 else if (action==2) then
  command='q'
  return
 end if
 read(*,*) command
 if (mwfn_session_active()==0.or.(trim(command)/='q'.and.trim(command)/='-10')) return
 ! Reject process exit at the main menu, then consume only this block's remainder.
 do
  inquire(unit=input_unit,pos=position)
  if (position>=mwfn_session_end()) exit
  read(*,'(a)') discarded
 end do
 rejected=1
end do
end subroutine
end module
