execute_process(COMMAND "${PROGRAM}" "${CASES}/input.txt"
  RESULT_VARIABLE status OUTPUT_VARIABLE actual ERROR_VARIABLE diagnostic)
if(NOT status EQUAL 0)
  message(FATAL_ERROR "Case runner failed (${status}): ${diagnostic}")
endif()
file(READ "${CASES}/expected.txt" expected)
if(NOT actual STREQUAL expected)
  message(FATAL_ERROR "Parity mismatch. Expected:\n${expected}\nActual:\n${actual}")
endif()
