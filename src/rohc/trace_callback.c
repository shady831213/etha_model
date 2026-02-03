#include <stdarg.h>
#include <stdio.h>
typedef enum
{
	ROHC_TRACE_DEBUG = 0,   /**< Print debug traces */
	ROHC_TRACE_INFO = 1,    /**< Print info (or lower) traces */
	ROHC_TRACE_WARNING = 2, /**< Print warning (or lower) traces */
	ROHC_TRACE_ERROR = 3,   /**< Print error (or lower) traces */
	ROHC_TRACE_LEVEL_MAX    /**< The maximum number of trace levels */
} rohc_trace_level_t;
typedef enum
{
	ROHC_TRACE_COMP = 0,    /**< Compressor traces */
	ROHC_TRACE_DECOMP = 1,  /**< Decompressor traces */
	ROHC_TRACE_ENTITY_MAX   /**< The maximum number of trace entities */
} rohc_trace_entity_t;
void print_rohc_traces(void *const priv_ctxt,
                              const rohc_trace_level_t level,
                              const rohc_trace_entity_t entity,
                              const int profile,
                              const char *const format,
                              ...)
{
	const char *level_descrs[] =
	{
		[ROHC_TRACE_DEBUG]   = "DEBUG",
		[ROHC_TRACE_INFO]    = "INFO",
		[ROHC_TRACE_WARNING] = "WARNING",
		[ROHC_TRACE_ERROR]   = "ERROR"
	};
	va_list args;

	fprintf(stdout, "[%s] ", level_descrs[level]);
	va_start(args, format);
	vfprintf(stdout, format, args);
	va_end(args);

}
