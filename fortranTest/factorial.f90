subroutine factorial(n, fact)
  implicit none
  integer, intent(in) :: n
  integer, intent(inout) :: fact
  integer :: i

  fact = 1
  if (n == 0) then
    fact = 1
  else
    do i=2,n
      fact = fact * i
    end do
  end if
  return
end subroutine factorial

integer function fact(n)
  implicit none
  integer, intent(in) :: n
  integer :: i

  fact = 1
  if (n == 0) then
    fact = 1
  else
    do i=2,n
      fact = fact * i
    end do
  end if
end function fact

program main
  implicit none
  integer :: n
  integer :: f
  integer :: fact

  read *, n

  call factorial(n, f)

  print *, f, fact(n)

end program main
