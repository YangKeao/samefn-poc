/* Independent MPFR nearest-even binary64 oracle, including gradual underflow. */
#include <inttypes.h>
#include <math.h>
#include <mpfr.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "samefn.h"
static uint64_t state=0x32545f4c8e9a1237ULL;
static uint64_t next(void){state^=state>>12;state^=state<<25;state^=state>>27;return state*0x2545f4914f6cdd1dULL;}
static uint64_t bits(double x){uint64_t u;memcpy(&u,&x,8);return u;}
static double value(uint64_t u){double x;memcpy(&x,&u,8);return x;}
int main(int argc,char**argv){
 if(argc<3)return 2;const char*name=argv[1];long n=strtol(argv[2],0,10);
 double(*kernel)(double)=!strcmp(name,"cos")?samefn_cos_v1:!strcmp(name,"tan")?samefn_tan_v1:samefn_exp_v1;
 int(*oracle)(mpfr_ptr,mpfr_srcptr,mpfr_rnd_t)=!strcmp(name,"cos")?mpfr_cos:!strcmp(name,"tan")?mpfr_tan:mpfr_exp;
 FILE*out=argc>3?fopen(argv[3],"w"):NULL;if(out)fprintf(out,"# MPFR binary64 nearest-even; independent expected bits for %s v1\n",name);
 mpfr_set_emin(-1073);mpfr_set_emax(1024);mpfr_t m;mpfr_init2(m,53);long count=0,diffs=0;
 double special[]={0.,-0.,1.,-1.,0.5,-0.5,3.141592653589793,1.5707963267948966,1e300,-1e300,0x1p-1074,-0x1p-1074,709.782712893384,710.,-745.1332191019411,-746.,INFINITY,-INFINITY,NAN};
 for(long i=0;i<n+(long)(sizeof(special)/sizeof(*special));i++){
  uint64_t u=next();double x=i<(long)(sizeof(special)/sizeof(*special))?special[i]:(i%4==0?value(u):i%4==1?((double)(u>>11)*0x1p-53*2-1)*10:i%4==2?((double)(u>>11)*0x1p-53*2-1)*750:((double)(u>>11)*0x1p-53*2-1)*1e12);
  mpfr_set_d(m,x,MPFR_RNDN);int inex=oracle(m,m,MPFR_RNDN);mpfr_subnormalize(m,inex,MPFR_RNDN);double want=mpfr_get_d(m,MPFR_RNDN);double got=kernel(x);count++;
  if(!((isnan(got)&&isnan(want))||bits(got)==bits(want))){if(diffs++<8)fprintf(stderr,"DIFF %s %a got %a want %a\n",name,x,got,want);}
  if(out)fprintf(out,"%016" PRIx64 "\t%016" PRIx64 "\n",bits(x),bits(want));
 }
 if(out)fclose(out);mpfr_clear(m);fprintf(stderr,"MPFR %s checked=%ld differences=%ld\n",name,count,diffs);return diffs?1:0;
}
