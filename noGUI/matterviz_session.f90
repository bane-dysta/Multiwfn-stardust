! Input/lifecycle adapter only. Called by a generated copy of the upstream entry.
module matterviz_session
use iso_c_binding
use iso_fortran_env, only: input_unit,output_unit
implicit none
logical :: session_enabled=.false.
logical :: input_open=.false.
interface
 type(c_ptr) function mwfn_session_commands() bind(C)
 import c_ptr
 end function
 integer(c_int) function mwfn_session_command_length() bind(C)
 import c_int
 end function
 subroutine mwfn_session_set_end(position) bind(C)
 import c_int64_t
 integer(c_int64_t),value :: position
 end subroutine
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
if (mwfn_session_initialize()<0) return
session_enabled=.true.
end subroutine

subroutine prepare_input()
character(c_char),pointer :: bytes(:)
character(len=:),allocatable :: line
integer :: n,i,start,finish,j,istat
integer(c_int64_t) :: boundary
n=mwfn_session_command_length()
call c_f_pointer(mwfn_session_commands(),bytes,[n])
close(input_unit)
open(unit=input_unit,status='scratch',access='stream',form='formatted',action='readwrite',iostat=istat)
if (istat/=0) error stop 'Cannot create managed input stream'
input_open=.true.
allocate(character(len=n) :: line)
start=1
do i=1,n
 if (bytes(i)/=c_new_line) cycle
 finish=i-1
 if (finish>=start) then
  if (bytes(finish)==c_carriage_return) finish=finish-1
 end if
 do j=start,finish
  line(j-start+1:j-start+1)=bytes(j)
 end do
 write(input_unit,'(a)') line(:finish-start+1)
 start=i+1
end do
flush(input_unit)
inquire(unit=input_unit,pos=boundary)
call mwfn_session_set_end(boundary)
rewind(input_unit)
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
 position=1
 if (input_open) inquire(unit=input_unit,pos=position)
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
 if (action==3) call prepare_input()
 if (action==1.or.action==2) then
  if (input_open) close(input_unit)
  input_open=.false.
 end if
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
