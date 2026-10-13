! Verify the runtime contract used by the optional menu input adapter.
program session_input_probe
use iso_fortran_env, only: input_unit,int64
implicit none
integer :: iteration,choice
integer(int64) :: boundary,position
real :: value
character(80) :: line
do iteration=1,3
 close(input_unit)
 open(unit=input_unit,status='scratch',access='stream',form='formatted',action='readwrite')
 write(input_unit,'(a)') '1000'
 write(input_unit,'(a)') '0.125'
 write(input_unit,'(a)') 'a path with spaces'
 flush(input_unit)
 inquire(unit=input_unit,pos=boundary)
 rewind(input_unit)
 read(*,*) choice
 read(*,*) value
 read(*,'(a)') line
 inquire(unit=input_unit,pos=position)
 if (choice/=1000.or.abs(value-0.125)>0.0001.or.trim(line)/='a path with spaces') stop 1
 if (position/=boundary) stop 2
end do
close(input_unit)
print *, 'session input probe passed'
end program
